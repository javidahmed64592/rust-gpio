//! Humidity Indicator LED Actuator Task Implementation

use actuators::IndicatorLedController;
use anyhow::Result;
use gpio_core::{Command, load_config};
use tokio::sync::mpsc;

/// Run the humidity indicator LED actuator task with command receiver channel
///
/// # Arguments
/// * `command_rx` - Channel to receive humidity LED commands from the controller
///
/// # Behavior
/// Processes SetHumidityLeds commands to display low/medium/high humidity levels
pub async fn run_humidity_led_actuator(mut command_rx: mpsc::Receiver<Command>) -> Result<()> {
    // Load config to get humidity LED pins
    let config = load_config("config/config.yaml")?;
    let humidity_config = &config.gpio.led.humidity;

    // Initialize humidity indicator LED controller
    let mut leds = IndicatorLedController::new(
        humidity_config.low_pin,
        humidity_config.medium_pin,
        humidity_config.high_pin,
        "Humidity",
    )?;

    println!("[Humidity LEDs] Ready to receive commands!");

    // Process commands from the channel
    while let Some(command) = command_rx.recv().await {
        match command {
            Command::SetHumidityLeds { low, medium, high, brightness } => {
                leds.set_indicators(low, medium, high, brightness);
            }
            Command::IndicatorLedsOff => {
                leds.set_all_off();
            }
            _ => {} // Ignore non-humidity LED commands
        }
    }

    // Clean shutdown - turn all LEDs off
    leds.set_all_off();
    println!("[Humidity LEDs] Shutting down...");

    Ok(())
}
