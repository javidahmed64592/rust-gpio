//! Controller Implementation
//!
//! Central decision-making logic for the GPIO system.

use anyhow::Result;
use gpio_core::{
    Command, Event, LightingMode, LightingPatternConfig, RgbColor, SystemState, load_config,
};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

/// Run the controller with event receiver and command sender channels
///
/// # Arguments
/// * `event_rx` - Channel to receive events from sensors
/// * `led_tx` - Channel to send commands to RGB LED actuator
/// * `lcd_tx` - Channel to send commands to LCD display
/// * `pattern_tx` - Channel to send pattern updates (pattern_index, brightness, paused)
/// * `temp_led_tx` - Channel to send commands to temperature indicator LEDs
/// * `humidity_led_tx` - Channel to send commands to humidity indicator LEDs
///
/// # Behavior
/// Central brain that processes events, maintains state, and generates commands
pub async fn run_controller(
    mut event_rx: mpsc::Receiver<Event>,
    led_tx: mpsc::Sender<Command>,
    lcd_tx: mpsc::Sender<Command>,
    pattern_tx: mpsc::Sender<(usize, u8, bool)>,
    temp_led_tx: mpsc::Sender<Command>,
    humidity_led_tx: mpsc::Sender<Command>,
) -> Result<()> {
    // Load config
    let config = load_config("config/config.yaml")?;

    // Initialize system state with config values
    let mut state = SystemState::default();
    state.brightness_level = config.system.default_brightness;
    state.current_pattern_index = config
        .system
        .default_pattern_index
        .min(config.lighting_patterns.len().saturating_sub(1));

    // Set initial brightness index to match default brightness
    if let Some(idx) = config
        .system
        .brightness_levels
        .iter()
        .position(|&b| b == config.system.default_brightness)
    {
        state.brightness_index = idx;
    }

    // Send initial pattern to executor (not paused initially)
    if let Err(e) = pattern_tx
        .send((state.current_pattern_index, state.brightness_level, false))
        .await
    {
        eprintln!("[Controller] Failed to send initial pattern: {}", e);
    }

    // Send initial pattern name to LCD line 0
    if !config.lighting_patterns.is_empty() {
        let initial_pattern_name = config.lighting_patterns[state.current_pattern_index].name();
        if let Err(e) = lcd_tx
            .send(Command::DisplayText {
                line: 0,
                text: initial_pattern_name,
            })
            .await
        {
            eprintln!("[Controller] Failed to send initial pattern to LCD: {}", e);
        }
    }

    println!("[Controller] Ready!");
    println!(
        "[Controller] Presence timeout: {} seconds",
        config.system.presence_timeout_secs
    );
    println!(
        "[Controller] Brightness levels: {:?}",
        config.system.brightness_levels
    );
    println!(
        "[Controller] Lighting patterns: {} available",
        config.lighting_patterns.len()
    );

    if !config.lighting_patterns.is_empty() {
        let initial_pattern_name = &config.lighting_patterns[state.current_pattern_index].name();
        println!(
            "[Controller] Starting with pattern {}/{}: {}",
            state.current_pattern_index + 1,
            config.lighting_patterns.len(),
            initial_pattern_name
        );
    }

    // Create a timer to check for presence timeout every 5 seconds
    let mut timeout_checker = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            // Handle incoming events from sensors
            Some(event) = event_rx.recv() => {
                println!("[Controller] Event received: {:?}", event);

                // Handle the event and get commands to send
                let commands = handle_event(event, &mut state, &config, &pattern_tx).await;

                // Send commands to actuators
                for command in commands {
                    println!("[Controller] Sending command: {:?}", command);

                    // Send LED commands to LED actuator (now RGB LED)
                    if matches!(command, Command::RgbLedOn | Command::RgbLedOff | Command::SetRgbBrightness(_) | Command::SetRgbColor(_) | Command::RgbLedBlinkError(_)) {
                        if let Err(e) = led_tx.send(command.clone()).await {
                            eprintln!("[Controller] Error sending to RGB LED: {}", e);
                        }
                    }

                    // Send display control commands to LCD
                    if matches!(command, Command::DisplayOn | Command::DisplayOff | Command::DisplayText { .. }) {
                        if let Err(e) = lcd_tx.send(command.clone()).await {
                            eprintln!("[Controller] Error sending to LCD: {}", e);
                        }
                    }

                    // Send temperature indicator LED commands
                    if matches!(command, Command::SetTemperatureLeds { .. }) {
                        if let Err(e) = temp_led_tx.send(command.clone()).await {
                            eprintln!("[Controller] Error sending to temperature LEDs: {}", e);
                        }
                    }

                    // Send humidity indicator LED commands
                    if matches!(command, Command::SetHumidityLeds { .. }) {
                        if let Err(e) = humidity_led_tx.send(command.clone()).await {
                            eprintln!("[Controller] Error sending to humidity LEDs: {}", e);
                        }
                    }

                    // Send IndicatorLedsOff to both temperature and humidity LED channels
                    if matches!(command, Command::IndicatorLedsOff) {
                        if let Err(e) = temp_led_tx.send(command.clone()).await {
                            eprintln!("[Controller] Error sending to temperature LEDs: {}", e);
                        }
                        if let Err(e) = humidity_led_tx.send(command).await {
                            eprintln!("[Controller] Error sending to humidity LEDs: {}", e);
                        }
                    }
                }
            }

            // Check for presence timeout every 5 seconds
            _ = timeout_checker.tick() => {
                if state.presence_detected {
                    if let Some(last_motion) = state.last_motion_time {
                        let elapsed = last_motion.elapsed().as_secs();

                        if elapsed >= config.system.presence_timeout_secs {
                            println!(
                                "[Controller] Presence timeout reached ({} seconds since last motion)",
                                elapsed
                            );
                            state.presence_detected = false;

                            // In Automatic mode, turn LED and LCD off when presence expires
                            if state.lighting_mode == LightingMode::Automatic {
                                println!("[Controller] Automatic mode: turning LED and LCD OFF");
                                if let Err(e) = led_tx.send(Command::RgbLedOff).await {
                                    eprintln!("[Controller] Error sending RgbLedOff command: {}", e);
                                }
                                if let Err(e) = lcd_tx.send(Command::DisplayOff).await {
                                    eprintln!("[Controller] Error sending DisplayOff command: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            // Channel closed, exit
            else => {
                break;
            }
        }
    }

    println!("[Controller] Shutting down...");
    Ok(())
}

/// Process a sensor event and update system state
/// Returns commands to be sent to actuators
///
/// # Arguments
/// * `event` - The sensor event to process
/// * `state` - Mutable reference to system state
/// * `config` - System configuration with brightness levels
/// * `pattern_tx` - Channel to send pattern updates (pattern_index, brightness, paused)
async fn handle_event(
    event: Event,
    state: &mut SystemState,
    config: &gpio_core::Config,
    pattern_tx: &mpsc::Sender<(usize, u8, bool)>,
) -> Vec<Command> {
    let mut commands = Vec::new();

    // Helper to check if current pattern is EventDriven
    let is_event_driven = matches!(
        config.lighting_patterns.get(state.current_pattern_index),
        Some(LightingPatternConfig::EventDriven)
    );

    match event {
        Event::MotionDetected => {
            println!("[Controller] Motion detected!");

            // Update presence state and timestamp
            let was_present = state.presence_detected;
            state.presence_detected = true;
            state.last_motion_time = Some(Instant::now());

            // Handle motion in Automatic mode
            if state.lighting_mode == LightingMode::Automatic {
                if !was_present {
                    println!("[Controller] Presence started - turning displays ON");
                    commands.push(Command::DisplayOn);

                    // For EventDriven patterns, controller manages LED color
                    if is_event_driven {
                        commands.push(Command::SetRgbColor(RgbColor::green()));
                        commands.push(Command::RgbLedOn);
                    }
                    // For other patterns, pattern executor handles LED colors
                } else if is_event_driven {
                    println!("[Controller] Presence extended - flash blue to indicate motion");
                    // Flash blue momentarily to indicate motion was detected
                    commands.push(Command::SetRgbColor(RgbColor::blue()));
                }
            }
        }

        Event::LightingModeTogglePressed => {
            // Toggle between Automatic and Manual Override
            state.lighting_mode = match state.lighting_mode {
                LightingMode::Automatic => {
                    println!(
                        "[Controller] Switching to Manual Override - turning LED, LCD, and indicator LEDs OFF"
                    );
                    commands.push(Command::RgbLedOff);
                    commands.push(Command::DisplayOff);
                    commands.push(Command::IndicatorLedsOff);

                    // Disable indicator LEDs
                    state.indicator_leds_enabled = false;

                    // Pause pattern executor updates
                    if let Err(e) = pattern_tx
                        .send((state.current_pattern_index, state.brightness_level, true))
                        .await
                    {
                        eprintln!("[Controller] Failed to send pattern pause: {}", e);
                    }

                    LightingMode::ManualOverride
                }
                LightingMode::ManualOverride => {
                    println!("[Controller] Switching to Automatic mode - enabling all displays");

                    // Enable indicator LEDs
                    state.indicator_leds_enabled = true;

                    // Resume pattern executor updates
                    if let Err(e) = pattern_tx
                        .send((state.current_pattern_index, state.brightness_level, false))
                        .await
                    {
                        eprintln!("[Controller] Failed to send pattern resume: {}", e);
                    }

                    // If presence is detected, turn displays back on
                    if state.presence_detected {
                        println!("[Controller] Presence detected - turning displays ON");
                        commands.push(Command::DisplayOn);

                        // For EventDriven patterns, also manually turn on LED with green
                        // (other patterns are handled by pattern executor)
                        if is_event_driven {
                            commands.push(Command::SetRgbColor(RgbColor::green()));
                            commands.push(Command::RgbLedOn);
                        }
                    }

                    LightingMode::Automatic
                }
            };
            println!("[Controller] Lighting mode now: {:?}", state.lighting_mode);
        }

        Event::BrightnessButtonPressed => {
            // Cycle through configurable brightness levels
            let brightness_levels = &config.system.brightness_levels;

            if !brightness_levels.is_empty() {
                // Move to next brightness level
                state.brightness_index = (state.brightness_index + 1) % brightness_levels.len();
                state.brightness_level = brightness_levels[state.brightness_index];

                println!(
                    "[Controller] Brightness adjusted to: {}% (level {}/{})",
                    state.brightness_level,
                    state.brightness_index + 1,
                    brightness_levels.len()
                );

                // Send brightness update to pattern executor (preserve pause state)
                let paused = state.lighting_mode == LightingMode::ManualOverride;
                if let Err(e) = pattern_tx
                    .send((state.current_pattern_index, state.brightness_level, paused))
                    .await
                {
                    eprintln!("[Controller] Failed to send brightness update: {}", e);
                }

                // Also update LED brightness directly for EventDriven patterns
                if is_event_driven && state.presence_detected {
                    commands.push(Command::SetRgbBrightness(state.brightness_level));
                }
            }
        }

        Event::PatternCyclePressed => {
            // Cycle to next pattern
            if !config.lighting_patterns.is_empty() {
                state.current_pattern_index =
                    (state.current_pattern_index + 1) % config.lighting_patterns.len();

                let pattern_name = &config.lighting_patterns[state.current_pattern_index].name();
                println!(
                    "[Controller] Cycling to pattern {}/{}: {}",
                    state.current_pattern_index + 1,
                    config.lighting_patterns.len(),
                    pattern_name
                );

                // Send pattern update to executor (preserve pause state)
                let paused = state.lighting_mode == LightingMode::ManualOverride;
                if let Err(e) = pattern_tx
                    .send((state.current_pattern_index, state.brightness_level, paused))
                    .await
                {
                    eprintln!("[Controller] Failed to send pattern update: {}", e);
                }

                // Update LCD line 0 with pattern name
                commands.push(Command::DisplayText {
                    line: 0,
                    text: pattern_name.clone(),
                });
            }
        }

        Event::EnvironmentReading {
            temperature_celsius,
            humidity_percent,
        } => {
            println!(
                "[Controller] Environment: {:.1}\u{00b0}C, {:.1}% RH",
                temperature_celsius, humidity_percent
            );

            // Determine temperature indicator LED state based on thresholds
            let temp_config = &config.gpio.led.temperature;
            let (temp_low, temp_medium, temp_high) =
                if temperature_celsius < temp_config.medium_threshold {
                    (true, false, false) // Low temperature
                } else if temperature_celsius < temp_config.high_threshold {
                    (false, true, false) // Medium temperature
                } else {
                    (false, false, true) // High temperature
                };

            // Use current brightness level for indicator LEDs, or 0 if disabled
            let indicator_brightness = if state.indicator_leds_enabled {
                state.brightness_level
            } else {
                0
            };

            commands.push(Command::SetTemperatureLeds {
                low: temp_low,
                medium: temp_medium,
                high: temp_high,
                brightness: indicator_brightness,
            });

            // Determine humidity indicator LED state based on thresholds
            let humidity_config = &config.gpio.led.humidity;
            let (hum_low, hum_medium, hum_high) =
                if humidity_percent < humidity_config.medium_threshold {
                    (true, false, false) // Low humidity
                } else if humidity_percent < humidity_config.high_threshold {
                    (false, true, false) // Medium humidity
                } else {
                    (false, false, true) // High humidity
                };

            commands.push(Command::SetHumidityLeds {
                low: hum_low,
                medium: hum_medium,
                high: hum_high,
                brightness: indicator_brightness,
            });

            // Update LCD line 1 with temperature and humidity
            commands.push(Command::DisplayText {
                line: 1,
                text: format!("{:.1}C {:.0}%RH", temperature_celsius, humidity_percent),
            });
        }
    }

    commands
}
