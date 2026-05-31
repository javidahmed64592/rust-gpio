//! Generic LED Test Binary
//!
//! Test any LED by specifying its GPIO pin number via CLI arguments.
//! This allows testing that LEDs are wired correctly.

use anyhow::{Context, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <gpio_pin_number>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} 26    # Test LED on GPIO 26 (PIR LED)", args[0]);
        eprintln!("  {} 17    # Test LED on GPIO 17", args[0]);
        std::process::exit(1);
    }

    let pin_number: u8 = args[1]
        .parse()
        .context("GPIO pin number must be a valid integer (0-27)")?;

    println!("=== LED Test ===");
    println!("Testing LED on GPIO pin: {}", pin_number);
    println!("\nThis test will:");
    println!("  • Initialize the GPIO pin as output");
    println!("  • Blink the LED 5 times (500ms on, 500ms off)");
    println!("  • Turn the LED off at the end");
    println!("\nPress Ctrl+C to exit early\n");

    // Initialize LED
    let mut led = actuators::LedController::new(pin_number, "LED Test")?;

    // Ensure LED starts off
    led.turn_off();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Blink 5 times
    for i in 1..=5 {
        println!("\nBlink {}/5", i);
        led.turn_on();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        led.turn_off();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("\n✓ LED test complete!");
    println!("If the LED blinked 5 times, the wiring is correct.\n");

    Ok(())
}
