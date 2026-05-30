//! RGB LED Actuator Task Implementation

use actuators::RgbLedController;
use anyhow::Result;
use gpio_core::{Command, load_config};
use tokio::sync::mpsc;

/// Run the RGB LED actuator task with a command receiver channel
///
/// # Arguments
/// * `command_rx` - Channel to receive RGB LED commands from the controller
///
/// # Behavior
/// Processes RGB LED commands (on/off/color/brightness/blink) until channel closes
pub async fn run_rgb_led_actuator(mut command_rx: mpsc::Receiver<Command>) -> Result<()> {
    // Load config to get RGB LED pins
    let config = load_config("config/config.yaml")?;
    let rgb_pins = &config.gpio.rgb_led.system;

    // Initialize RGB LED controller
    let mut led = RgbLedController::new(
        rgb_pins.red_pin,
        rgb_pins.green_pin,
        rgb_pins.blue_pin,
        "System RGB LED",
    )?;

    // Ensure LED starts off
    led.turn_off();
    println!("[RGB LED] Ready to receive commands!");

    // Process commands from the channel
    while let Some(command) = command_rx.recv().await {
        match command {
            Command::RgbLedOn => led.turn_on(),
            Command::RgbLedOff => led.turn_off(),
            Command::SetRgbColor(color) => led.set_color(color),
            Command::SetRgbBrightness(level) => led.set_brightness(level),
            Command::RgbLedBlinkError(times) => led.blink_error(times),
            _ => {} // Ignore non-RGB-LED commands
        }
    }

    // Clean shutdown - turn off LED
    led.turn_off();
    println!("[RGB LED] Shutting down...");

    Ok(())
}
