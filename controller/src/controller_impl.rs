//! Controller Implementation
//!
//! Central decision-making logic for the GPIO system.

use anyhow::Result;
use gpio_core::{Command, Event, LightingMode, SystemState, load_config};
use std::time::Instant;
use tokio::sync::mpsc;

/// Run the controller with event receiver and command sender channels
pub async fn run_controller(
    mut event_rx: mpsc::Receiver<Event>,
    led_tx: mpsc::Sender<Command>,
) -> Result<()> {
    // Load config
    let config = load_config("config/config.yaml")?;

    // Initialize system state
    let mut state = SystemState::default();
    state.brightness_level = config.system.default_brightness;
    println!("[Controller] Ready!");

    // Process events from sensors
    while let Some(event) = event_rx.recv().await {
        println!("[Controller] Event received: {:?}", event);

        // Handle the event and get commands to send
        let commands = handle_event(event, &mut state, &config);

        // Send commands to actuators
        for command in commands {
            println!("[Controller] Sending command: {:?}", command);
            if let Err(e) = led_tx.send(command).await {
                eprintln!("[Controller] Error sending command: {}", e);
            }
        }
    }

    println!("[Controller] Shutting down...");
    Ok(())
}

/// Process a sensor event and update system state
/// Returns commands to be sent to actuators
fn handle_event(event: Event, state: &mut SystemState, config: &gpio_core::Config) -> Vec<Command> {
    let mut commands = Vec::new();

    match event {
        Event::MotionDetected => {
            println!("[Controller] Motion detected!");
            state.presence_detected = true;
            state.last_motion_time = Some(Instant::now());

            // In Automatic mode, turn LED on when motion detected
            if state.lighting_mode == LightingMode::Automatic {
                println!("[Controller] Automatic mode: turning LED ON");
                commands.push(Command::LedOn);
                commands.push(Command::SetBrightness(state.brightness_level));
            }
        }

        Event::MotionExpired => {
            println!("[Controller] Motion expired!");

            // Check if timeout has elapsed
            if let Some(last_motion) = state.last_motion_time {
                let elapsed = last_motion.elapsed().as_secs();
                if elapsed >= config.system.presence_timeout_secs {
                    println!(
                        "[Controller] Presence timeout reached ({} seconds)",
                        elapsed
                    );
                    state.presence_detected = false;

                    // In Automatic mode, turn LED off when presence expires
                    if state.lighting_mode == LightingMode::Automatic {
                        println!("[Controller] Automatic mode: turning LED OFF");
                        commands.push(Command::LedOff);
                    }
                }
            }
        }

        Event::LightingModeTogglePressed => {
            // Toggle between Automatic and Manual Override
            state.lighting_mode = match state.lighting_mode {
                LightingMode::Automatic => LightingMode::ManualOverride,
                LightingMode::ManualOverride => LightingMode::Automatic,
            };
            println!(
                "[Controller] Lighting mode toggled to: {:?}",
                state.lighting_mode
            );
        }

        Event::BrightnessButtonPressed => {
            // Cycle through brightness levels: 25%, 50%, 75%, 100%
            state.brightness_level = match state.brightness_level {
                0..=25 => 50,
                26..=50 => 75,
                51..=75 => 100,
                _ => 25,
            };
            println!(
                "[Controller] Brightness adjusted to: {}%",
                state.brightness_level
            );

            // Update LED brightness if it's currently on
            if state.presence_detected {
                commands.push(Command::SetBrightness(state.brightness_level));
            }
        }

        Event::TiltUpdated { pitch, roll } => {
            println!(
                "[Controller] Tilt update: pitch={:.2}, roll={:.2}",
                pitch, roll
            );
            // TODO: Future implementation - could trigger alerts or display updates
        }
    }

    commands
}
