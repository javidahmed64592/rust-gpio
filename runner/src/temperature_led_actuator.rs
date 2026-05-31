//! Temperature Indicator LED Actuator Task Implementation

use actuators::IndicatorLedController;
use anyhow::Result;
use gpio_core::{Command, load_config};
use tokio::sync::mpsc;

/// Run the temperature indicator LED actuator task with command receiver channel
///
/// # Arguments
/// * `command_rx` - Channel to receive temperature LED commands from the controller
///
/// # Behavior
/// Processes SetTemperatureLeds commands to display low/medium/high temperature levels
pub async fn run_temperature_led_actuator(mut command_rx: mpsc::Receiver<Command>) -> Result<()> {
    // Load config to get temperature LED pins
    let config = load_config("config/config.yaml")?;
    let temp_config = &config.gpio.led.temperature;

    // Initialize temperature indicator LED controller
    let mut leds = IndicatorLedController::new(
        temp_config.low_pin,
        temp_config.medium_pin,
        temp_config.high_pin,
        "Temperature",
    )?;

    println!("[Temperature LEDs] Ready to receive commands!");

    // Process commands from the channel
    while let Some(command) = command_rx.recv().await {
        match command {
            Command::SetTemperatureLeds { low, medium, high, brightness } => {
                leds.set_indicators(low, medium, high, brightness);
            }
            Command::IndicatorLedsOff => {
                leds.set_all_off();
            }
            _ => {} // Ignore non-temperature LED commands
        }
    }

    // Clean shutdown - turn all LEDs off
    leds.set_all_off();
    println!("[Temperature LEDs] Shutting down...");

    Ok(())
}
