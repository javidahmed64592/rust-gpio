//! Controller - The Brain of the GPIO System
//!
//! Standalone binary for testing the controller.

use anyhow::Result;
use gpio_core::Event;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // For standalone testing, create channels and send test events
    let (event_tx, event_rx) = mpsc::channel(32);
    let (led_tx, mut led_rx) = mpsc::channel(32);
    let (lcd_tx, mut lcd_rx) = mpsc::channel(32);
    let (pattern_tx, mut pattern_rx) = mpsc::channel::<(usize, u8, bool)>(32); // (pattern_index, brightness, paused)

    // Spawn the controller
    let controller_handle = tokio::spawn(async move {
        controller::run_controller(event_rx, led_tx, lcd_tx, pattern_tx).await
    });

    // Send some test events
    println!("\n=== Standalone Controller Test ===");
    println!("Sending test events...\n");

    event_tx.send(Event::MotionDetected).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Receive and print commands from both channels
    while let Ok(cmd) = led_rx.try_recv() {
        println!("LED command: {:?}", cmd);
    }
    while let Ok(cmd) = lcd_rx.try_recv() {
        println!("LCD command: {:?}", cmd);
    }
    while let Ok((idx, brightness, paused)) = pattern_rx.try_recv() {
        println!(
            "Pattern command: index={}, brightness={}%, paused={}",
            idx, brightness, paused
        );
    }

    println!("\nTest complete. Press Ctrl+C to exit.");

    // Keep running
    tokio::signal::ctrl_c().await?;

    drop(event_tx);
    let _ = controller_handle.await;

    Ok(())
}
