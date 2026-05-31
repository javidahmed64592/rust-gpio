//! Generic Button Test Binary
//!
//! Test any button by specifying its GPIO pin number via CLI arguments.
//! This allows testing that buttons are wired correctly and detecting button presses.

use anyhow::{Context, Result};
use sensors::ButtonController;
use std::env;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <gpio_pin_number>", args[0]);
        eprintln!("\nExample:");
        eprintln!(
            "  {} 21    # Test button on GPIO 21 (override button)",
            args[0]
        );
        eprintln!(
            "  {} 20    # Test button on GPIO 20 (brightness button)",
            args[0]
        );
        std::process::exit(1);
    }

    let pin_number: u8 = args[1]
        .parse()
        .context("GPIO pin number must be a valid integer (0-27)")?;

    println!("=== Button Test ===");
    println!("Testing button on GPIO pin: {}", pin_number);
    println!("\nThis test will:");
    println!("  • Initialize the GPIO pin with internal pull-up resistor");
    println!("  • Poll for button presses (falling edge: HIGH → LOW)");
    println!("  • Print a message each time the button is pressed");
    println!("  • Use 300ms debounce to prevent double-triggering");
    println!("\nPress Ctrl+C to exit\n");

    // Initialize button controller
    let mut button = ButtonController::new(pin_number, "Button Test")?;

    let mut press_count = 0;
    const DEBOUNCE_MS: u64 = 300;

    // Poll for button presses
    loop {
        if button.is_pressed() {
            press_count += 1;
            println!("✓ Button press #{} detected!", press_count);

            // Debounce
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;
        }

        // Poll every 50ms
        sleep(Duration::from_millis(50)).await;
    }
}
