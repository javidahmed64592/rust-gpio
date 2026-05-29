//! LED Actuator Binary
//!
//! Standalone binary for testing the LED actuator.

use anyhow::Result;
use gpio_core::Command;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // For standalone testing, create a channel and send test commands
    let (tx, rx) = mpsc::channel(32);

    // Spawn the LED actuator
    let led_handle = tokio::spawn(async move { actuators::run_led_actuator(rx).await });

    // Send some test commands
    println!("\n=== Standalone LED Test ===");
    println!("Sending test commands...\n");

    tx.send(Command::LedOn).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    tx.send(Command::LedOff).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("\nTest complete. Press Ctrl+C to exit.");

    // Keep running
    tokio::signal::ctrl_c().await?;

    // Close channel and wait for cleanup
    drop(tx);
    let _ = led_handle.await;

    Ok(())
}
