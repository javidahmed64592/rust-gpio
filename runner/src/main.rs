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

    // Command channel: controller -> PIR LED actuator
    let (pir_led_cmd_tx, pir_led_cmd_rx) = mpsc::channel(32);

    println!("Spawning components...");

    // Spawn controller task
    let controller_handle = tokio::spawn(async move {
        if let Err(e) = controller::run_controller(event_rx, pir_led_cmd_tx).await {
            eprintln!("Controller error: {}", e);
        }
    });
    println!("  ✓ Controller spawned");

    // Spawn PIR LED actuator task
    let pir_led_handle = tokio::spawn(async move {
        if let Err(e) = actuators::run_pir_led_actuator(pir_led_cmd_rx).await {
            eprintln!("PIR LED actuator error: {}", e);
        }
    });
    println!("  ✓ PIR LED actuator spawned");

    // Clone event_tx for each sensor
    let pir_event_tx = event_tx.clone();
    let override_event_tx = event_tx.clone();
    let brightness_event_tx = event_tx;

    // Spawn PIR sensor task
    let pir_handle = tokio::spawn(async move {
        if let Err(e) = sensors::run_pir_sensor(pir_event_tx).await {
            eprintln!("PIR sensor error: {}", e);
        }
    });
    println!("  ✓ PIR sensor spawned");

    // Spawn lighting override button task
    let override_handle = tokio::spawn(async move {
        if let Err(e) = sensors::run_lighting_override_button(override_event_tx).await {
            eprintln!("Lighting override button error: {}", e);
        }
    });
    println!("  ✓ Lighting override button spawned");

    // Spawn brightness button task
    let brightness_handle = tokio::spawn(async move {
        if let Err(e) = sensors::run_brightness_button(brightness_event_tx).await {
            eprintln!("Brightness button error: {}", e);
        }
    });
    println!("  ✓ Brightness button spawned");

    println!("\n=== System Ready ===");
    println!("All components running.");
    println!("");
    println!("Controls:");
    println!("  • PIR sensor: Wave hand to trigger motion detection");
    println!("  • Override button (GPIO 21): Toggle Automatic/Manual mode");
    println!("  • Brightness button (GPIO 20): Cycle brightness (25% → 50% → 75% → 100%)");
    println!("\nPress Ctrl+C to shut down\n");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\n\nShutdown signal received...");
    println!("Stopping all tasks...");

    // Wait for tasks to finish gracefully
    let _ = tokio::join!(
        pir_led_handle,
        controller_handle,
        pir_handle,
        override_handle,
        brightness_handle
    );

    println!("\nGPIO System shut down cleanly.");

    Ok(())
}
