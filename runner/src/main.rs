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

mod brightness_button;
mod lcd_display;
mod lighting_override_button;
mod pir_sensor;
mod rgb_led_actuator;

use anyhow::Result;
use gpio_core::Event;
use tokio::sync::{broadcast, mpsc};

use brightness_button::run_brightness_button;
use lcd_display::run_lcd_display;
use lighting_override_button::run_lighting_override_button;
use pir_sensor::run_pir_sensor;
use rgb_led_actuator::run_rgb_led_actuator;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== GPIO System Runner ===");
    println!("Bootstrapping event-driven GPIO system...");

    // Create communication channels
    // Event channel: sensors -> controller
    let (event_tx, event_rx) = mpsc::channel::<Event>(32);

    // Command channel: controller -> RGB LED actuator
    let (rgb_led_cmd_tx, rgb_led_cmd_rx) = mpsc::channel(32);

    // Command channel: controller -> LCD actuator
    let (lcd_cmd_tx, lcd_cmd_rx) = mpsc::channel(32);

    // Shutdown signal channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    println!("Spawning components...");

    // Clone event_tx for each sensor
    let pir_event_tx = event_tx.clone();
    let override_event_tx = event_tx.clone();
    let brightness_event_tx = event_tx.clone();

    // Clone shutdown receiver for each task
    let pir_shutdown = shutdown_tx.subscribe();
    let override_shutdown = shutdown_tx.subscribe();
    let brightness_shutdown = shutdown_tx.subscribe();
    let controller_shutdown = shutdown_tx.subscribe();
    let led_shutdown = shutdown_tx.subscribe();
    let lcd_shutdown = shutdown_tx.subscribe();

    // === SENSORS ===

    // Spawn PIR sensor task
    let pir_handle = tokio::spawn(async move {
        let mut shutdown = pir_shutdown;
        tokio::select! {
            result = run_pir_sensor(pir_event_tx) => {
                if let Err(e) = result {
                    eprintln!("PIR sensor error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[PIR] Shutdown signal received");
            }
        }
    });
    println!("  ✓ PIR sensor spawned");

    // Spawn lighting override button task
    let override_handle = tokio::spawn(async move {
        let mut shutdown = override_shutdown;
        tokio::select! {
            result = run_lighting_override_button(override_event_tx) => {
                if let Err(e) = result {
                    eprintln!("Lighting override button error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Lighting Override] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Lighting override button spawned");

    // Spawn brightness button task
    let brightness_handle = tokio::spawn(async move {
        let mut shutdown = brightness_shutdown;
        tokio::select! {
            result = run_brightness_button(brightness_event_tx) => {
                if let Err(e) = result {
                    eprintln!("Brightness button error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Brightness Button] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Brightness button spawned");

    // === CONTROLLER ===

    // Clone command senders so we can use them for shutdown
    let shutdown_led_tx = rgb_led_cmd_tx.clone();
    let shutdown_lcd_tx = lcd_cmd_tx.clone();

    // Spawn controller task
    let controller_handle = tokio::spawn(async move {
        let mut shutdown = controller_shutdown;
        tokio::select! {
            result = controller::run_controller(event_rx, rgb_led_cmd_tx, lcd_cmd_tx) => {
                if let Err(e) = result {
                    eprintln!("Controller error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Controller] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Controller spawned");

    // === ACTUATORS ===

    // Spawn RGB LED actuator task
    let rgb_led_handle = tokio::spawn(async move {
        let mut shutdown = led_shutdown;
        tokio::select! {
            result = run_rgb_led_actuator(rgb_led_cmd_rx) => {
                if let Err(e) = result {
                    eprintln!("RGB LED actuator error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[RGB LED] Shutdown signal received");
            }
        }
    });
    println!("  ✓ RGB LED actuator spawned");

    // Spawn LCD display task
    let lcd_handle = tokio::spawn(async move {
        let mut shutdown = lcd_shutdown;
        tokio::select! {
            result = run_lcd_display(lcd_cmd_rx) => {
                if let Err(e) = result {
                    eprintln!("LCD display error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[LCD] Shutdown signal received");
            }
        }
    });
    println!("  ✓ LCD display spawned");

    println!("\n=== System Ready ===");
    println!("All components running.");
    println!("");
    println!("Controls:");
    println!("  • PIR sensor: Wave hand to trigger motion detection");
    println!("  • Override button: Toggle Automatic/Manual mode");
    println!("  • Brightness button: Cycle brightness (25% → 50% → 75% → 100%)");
    println!("\nPress Ctrl+C to shut down\n");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\n\nShutdown signal received...");
    println!("Stopping all tasks...");

    // Turn off RGB LED and LCD before shutting down
    println!("Turning off RGB LED and LCD...");
    let _ = shutdown_led_tx.send(gpio_core::Command::RgbLedOff).await;
    let _ = shutdown_lcd_tx.send(gpio_core::Command::DisplayOff).await;
    let _ = shutdown_lcd_tx.send(gpio_core::Command::ClearDisplay).await;

    // Give actuators time to process shutdown commands
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Broadcast shutdown signal to all tasks
    let _ = shutdown_tx.send(());

    // Drop the main event_tx to ensure controller can exit after processing remaining events
    drop(event_tx);

    // Wait for all tasks to finish gracefully
    let _ = tokio::join!(
        pir_handle,
        override_handle,
        brightness_handle,
        controller_handle,
        rgb_led_handle,
        lcd_handle
    );

    println!("\nGPIO System shut down cleanly.");

    Ok(())
}
