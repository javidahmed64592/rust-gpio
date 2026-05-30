//! PIR LED Actuator Implementation Module

use anyhow::Result;
use gpio_core::{Command, load_config};
use tokio::sync::mpsc;

use crate::led_controller::LedController;

/// Run the PIR LED actuator with a command receiver channel
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
            _ => {}
        }
    }

    // Clean shutdown - turn off LED
    led.turn_off();
    println!("[PIR LED] Shutting down...");

    Ok(())
}
