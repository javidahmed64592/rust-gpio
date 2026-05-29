//! Generic Button Test Binary
//!
//! Test any button by specifying its GPIO pin number via CLI arguments.
//! This allows testing that buttons are wired correctly and detecting button presses.

use anyhow::{Context, Result};
use gpio_core::Event;
use std::env;
use tokio::sync::mpsc;

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

    // Create a channel for testing
    let (event_tx, mut event_rx) = mpsc::channel::<Event>(32);

    // Spawn the button task
    let button_handle = tokio::spawn(async move {
        if let Err(e) = sensors::run_button(
            pin_number,
            Event::LightingModeTogglePressed, // Dummy event for testing
            "Button Test",
            event_tx,
        )
        .await
        {
            eprintln!("Button error: {}", e);
        }
    });

    // Listen for events and print them
    let event_handle = tokio::spawn(async move {
        let mut press_count = 0;
        while let Some(event) = event_rx.recv().await {
            press_count += 1;
            println!(
                "✓ Button press #{} detected! Event: {:?}",
                press_count, event
            );
        }
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\n\nShutdown signal received...");

    // Wait for tasks to finish
    let _ = tokio::join!(button_handle, event_handle);

    println!("Button test complete!");
    Ok(())
}
