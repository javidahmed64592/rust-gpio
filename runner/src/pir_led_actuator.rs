//! PIR LED Actuator Task Implementation

use actuators::LedController;
use anyhow::Result;
use gpio_core::{Command, load_config};
use tokio::sync::mpsc;

/// Run the PIR LED actuator task with a command receiver channel
///
/// # Arguments
/// * `command_rx` - Channel to receive LED commands from the controller
///
/// # Behavior
/// Processes LED commands (on/off/brightness/blink) until channel closes
pub async fn run_pir_led_actuator(mut command_rx: mpsc::Receiver<Command>) -> Result<()> {
    // Load config to get PIR LED pin
    let config = load_config("config/config.yaml")?;
    let led_pin = config.gpio.led.pir_led_pin;

    // Initialize PIR LED controller
    let mut led = LedController::new(led_pin, "PIR LED")?;

    // Ensure LED starts off
    led.turn_off();
    println!("[PIR LED] Ready to receive commands!");

    // Process commands from the channel
    while let Some(command) = command_rx.recv().await {
        match command {
            Command::LedOn => led.turn_on(),
            Command::LedOff => led.turn_off(),
            Command::SetBrightness(level) => led.set_brightness(level),
            Command::LedBlinkError(times) => led.blink_error(times),
            _ => {} // Ignore non-LED commands
        }
    }

    // Clean shutdown - turn off LED
    led.turn_off();
    println!("[PIR LED] Shutting down...");

    Ok(())
}
