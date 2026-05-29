//! Runner - System Orchestrator
//!
//! Main executable that:
//! - Bootstraps the entire application
//! - Loads configuration
//! - Initializes communication channels
//! - Spawns all sensor tasks
//! - Spawns controller task
//! - Spawns all actuator tasks
//! - Wires everything together
//!
//! Contains minimal business logic - primarily dependency injection and orchestration.

use anyhow::Result;
use gpio_core::Event;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== GPIO System Runner ===");
    println!("Bootstrapping event-driven GPIO system...");

    // Create communication channels
    // Event channel: sensors -> controller
    let (event_tx, event_rx) = mpsc::channel::<Event>(32);

    // Command channel: controller -> LED actuator
    let (led_cmd_tx, led_cmd_rx) = mpsc::channel(32);

    println!("Spawning components...");

    // Spawn LED actuator task
    let led_handle = tokio::spawn(async move {
        if let Err(e) = actuators::run_led_actuator(led_cmd_rx).await {
            eprintln!("LED actuator error: {}", e);
        }
    });
    println!("  ✓ LED actuator spawned");

    // Spawn controller task
    let controller_handle = tokio::spawn(async move {
        if let Err(e) = controller::run_controller(event_rx, led_cmd_tx).await {
            eprintln!("Controller error: {}", e);
        }
    });
    println!("  ✓ Controller spawned");

    // TODO: Spawn PIR sensor task
    // let pir_handle = tokio::spawn(async move {
    //     if let Err(e) = sensors::run_pir_sensor(event_tx.clone()).await {
    //         eprintln!("PIR sensor error: {}", e);
    //     }
    // });

    println!("\n=== System Ready ===");
    println!("Waiting for sensor events...\n");
    println!("Press Ctrl+C to shut down\n");

    // Send a test event to demonstrate the system works
    println!("=== Sending test event to demonstrate system ===");
    event_tx.send(Event::MotionDetected).await?;
    println!("Sent MotionDetected event\n");

    // Wait a moment to see the LED turn on
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\n\nShutdown signal received...");
    println!("Stopping all components...");

    // Drop event_tx to signal the controller to exit
    drop(event_tx);

    // Wait for tasks to finish gracefully
    let _ = tokio::join!(led_handle, controller_handle);

    println!("\nGPIO System shut down cleanly.");

    Ok(())
}
