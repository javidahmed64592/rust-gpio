//! Controller Implementation
//!
//! Central decision-making logic for the GPIO system.

use anyhow::Result;
use gpio_core::{Command, Event, LightingMode, SystemState, load_config};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

/// Run the controller with event receiver and command sender channels
///
/// # Arguments
/// * `event_rx` - Channel to receive events from sensors
/// * `led_tx` - Channel to send commands to LED actuator
/// * `lcd_tx` - Channel to send commands to LCD display
///
/// # Behavior
/// Central brain that processes events, maintains state, and generates commands
pub async fn run_controller(
    mut event_rx: mpsc::Receiver<Event>,
    led_tx: mpsc::Sender<Command>,
    lcd_tx: mpsc::Sender<Command>,
) -> Result<()> {
    // Load config
    let config = load_config("config/config.yaml")?;

    // Initialize system state with config values
    let mut state = SystemState::default();
    state.brightness_level = config.system.default_brightness;

    // Set initial brightness index to match default brightness
    if let Some(idx) = config
        .system
        .brightness_levels
        .iter()
        .position(|&b| b == config.system.default_brightness)
    {
        state.brightness_index = idx;
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

    // Create a timer to check for presence timeout every 5 seconds
    let mut timeout_checker = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            // Handle incoming events from sensors
            Some(event) = event_rx.recv() => {
                println!("[Controller] Event received: {:?}", event);

                // Send event info to LCD (top row - line 0) - use static strings where possible
                let event_text = match &event {
                    Event::MotionDetected => "Motion Detected",
                    Event::LightingModeTogglePressed => "Mode Toggle",
                    Event::BrightnessButtonPressed => "Brightness Adj",
                };

                if let Err(e) = lcd_tx.send(Command::DisplayText {
                    line: 0,
                    text: event_text.to_string(),
                }).await {
                    eprintln!("[Controller] Error sending event to LCD: {}", e);
                }

                // Handle the event and get commands to send
                let commands = handle_event(event, &mut state, &config);

                // Send commands to actuators
                for command in commands {
                    println!("[Controller] Sending command: {:?}", command);

                    // Send LED commands to LED actuator
                    if matches!(command, Command::LedOn | Command::LedOff | Command::SetBrightness(_) | Command::LedBlinkError(_)) {
                        if let Err(e) = led_tx.send(command.clone()).await {
                            eprintln!("[Controller] Error sending to LED: {}", e);
                        }

                        // Also send command description to LCD (bottom row - line 1)
                        let cmd_text = match &command {
                            Command::LedOn => "LED: ON",
                            Command::LedOff => "LED: OFF",
                            Command::SetBrightness(level) => {
                                // Need to format, so allocate here
                                &format!("Brightness: {}%", level)
                            },
                            Command::LedBlinkError(times) => {
                                &format!("Error! Blink x{}", times)
                            },
                            _ => "",
                        };
                        if !cmd_text.is_empty() {
                            if let Err(e) = lcd_tx.send(Command::DisplayText {
                                line: 1,
                                text: cmd_text.to_string(),
                            }).await {
                                eprintln!("[Controller] Error sending command to LCD: {}", e);
                            }
                        }
                    }

                    // Send display control commands to LCD
                    if matches!(command, Command::DisplayOn | Command::DisplayOff) {
                        if let Err(e) = lcd_tx.send(command).await {
                            eprintln!("[Controller] Error sending to LCD: {}", e);
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
                                if let Err(e) = led_tx.send(Command::LedOff).await {
                                    eprintln!("[Controller] Error sending LedOff command: {}", e);
                                }
                                // Send command description to LCD
                                if let Err(e) = lcd_tx.send(Command::DisplayText {
                                    line: 1,
                                    text: "LED: OFF".to_string(),
                                }).await {
                                    eprintln!("[Controller] Error sending to LCD: {}", e);
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
fn handle_event(event: Event, state: &mut SystemState, config: &gpio_core::Config) -> Vec<Command> {
    let mut commands = Vec::new();

    match event {
        Event::MotionDetected => {
            println!("[Controller] Motion detected!");

            // Update presence state and timestamp
            let was_present = state.presence_detected;
            state.presence_detected = true;
            state.last_motion_time = Some(Instant::now());

            // In Automatic mode, turn LED and LCD on when motion detected
            if state.lighting_mode == LightingMode::Automatic {
                if !was_present {
                    println!("[Controller] Automatic mode: turning LED and LCD ON");
                    commands.push(Command::LedOn);
                    commands.push(Command::DisplayOn);
                } else {
                    println!("[Controller] Presence extended (LED already on)");
                }
            }
        }

        Event::LightingModeTogglePressed => {
            // Toggle between Automatic and Manual Override
            state.lighting_mode = match state.lighting_mode {
                LightingMode::Automatic => {
                    println!("[Controller] Switching to Manual Override - turning LED and LCD OFF");
                    commands.push(Command::LedOff);
                    commands.push(Command::DisplayOff);
                    LightingMode::ManualOverride
                }
                LightingMode::ManualOverride => {
                    println!("[Controller] Switching to Automatic mode");
                    // If presence is detected, turn LED and LCD back on
                    if state.presence_detected {
                        println!("[Controller] Presence detected - turning LED and LCD ON");
                        commands.push(Command::LedOn);
                        commands.push(Command::DisplayOn);
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

                // Update LED brightness if it's currently on
                if state.presence_detected {
                    commands.push(Command::SetBrightness(state.brightness_level));
                }
            }
        }
    }

    commands
}
