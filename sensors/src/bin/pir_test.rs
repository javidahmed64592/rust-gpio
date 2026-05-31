//! Generic PIR Sensor Test Binary
//!
//! Test any PIR sensor by specifying its GPIO pin number via CLI arguments.
//! This allows testing that PIR sensors are wired correctly and detecting motion.

use anyhow::{Context, Result};
use sensors::PirSensorController;
use std::env;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <gpio_pin_number>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} 16    # Test PIR sensor on GPIO 16", args[0]);
        std::process::exit(1);
    }

    let pin_number: u8 = args[1]
        .parse()
        .context("GPIO pin number must be a valid integer (0-27)")?;

    println!("=== PIR Sensor Test ===");
    println!("Testing PIR sensor on GPIO pin: {}", pin_number);
    println!("\nThis test will:");
    println!("  • Initialize the GPIO pin as input");
    println!("  • Poll for motion detection (LOW → HIGH transitions)");
    println!("  • Print a message each time motion is detected");
    println!("\nPress Ctrl+C to exit\n");

    // Initialize PIR sensor controller
    let mut pir = PirSensorController::new(pin_number, "PIR Test")?;

    println!("PIR sensor running. Wave your hand to test motion detection.\n");

    // Poll for motion
    loop {
        if let Some(event) = pir.check_motion() {
            println!("Event detected: {:?}", event);
        }

        // Poll every 100ms
        sleep(Duration::from_millis(100)).await;
    }
}
