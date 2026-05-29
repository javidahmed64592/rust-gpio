//! PIR Sensor Binary
//!
//! Standalone binary for testing the PIR sensor.

use anyhow::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n=== Standalone PIR Sensor Test ===");
    println!("Starting PIR sensor...\n");

    // For standalone testing, create a channel to receive events
    let (event_tx, mut event_rx) = mpsc::channel(32);

    // Spawn the PIR sensor
    let pir_handle = tokio::spawn(async move {
        if let Err(e) = sensors::run_pir_sensor(event_tx).await {
            eprintln!("PIR sensor error: {}", e);
        }
    });

    // Print events as they arrive
    let print_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            println!("Event received: {:?}", event);
        }
    });

    println!("PIR sensor running. Wave your hand to test motion detection.");
    println!("Press Ctrl+C to exit.\n");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\nShutting down PIR sensor...");

    let _ = tokio::join!(pir_handle, print_handle);

    Ok(())
}
