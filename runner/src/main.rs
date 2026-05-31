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
mod dht11_sensor;
mod humidity_led_actuator;
mod lcd_display;
mod lighting_override_button;
mod pattern_button;
mod pattern_executor;
mod pir_sensor;
mod rgb_led_actuator;
mod temperature_led_actuator;

use anyhow::Result;
use gpio_core::Event;
use tokio::sync::{broadcast, mpsc};

use brightness_button::run_brightness_button;
use dht11_sensor::run_dht11_sensor;
use humidity_led_actuator::run_humidity_led_actuator;
use lcd_display::run_lcd_display;
use lighting_override_button::run_lighting_override_button;
use pattern_button::run_pattern_button;
use pattern_executor::run_pattern_executor;
use pir_sensor::run_pir_sensor;
use rgb_led_actuator::run_rgb_led_actuator;
use temperature_led_actuator::run_temperature_led_actuator;

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

    // Pattern channel: controller -> pattern executor (pattern_index, brightness, paused)
    let (pattern_tx, pattern_rx) = mpsc::channel::<(usize, u8, bool)>(32);

    // Command channel: controller -> temperature indicator LEDs
    let (temp_led_cmd_tx, temp_led_cmd_rx) = mpsc::channel(32);

    // Command channel: controller -> humidity indicator LEDs
    let (humidity_led_cmd_tx, humidity_led_cmd_rx) = mpsc::channel(32);

    // Pattern LED channel: pattern executor -> RGB LED (merged with controller commands)
    let (pattern_led_tx, pattern_led_rx) = mpsc::channel(32);

    // Shutdown signal channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    println!("Spawning components...");

    // Clone event_tx for each sensor
    let pir_event_tx = event_tx.clone();
    let override_event_tx = event_tx.clone();
    let brightness_event_tx = event_tx.clone();
    let pattern_event_tx = event_tx.clone();
    let dht11_event_tx = event_tx.clone();

    // Clone shutdown receiver for each task
    let pir_shutdown = shutdown_tx.subscribe();
    let override_shutdown = shutdown_tx.subscribe();
    let brightness_shutdown = shutdown_tx.subscribe();
    let pattern_button_shutdown = shutdown_tx.subscribe();
    let dht11_shutdown = shutdown_tx.subscribe();
    let controller_shutdown = shutdown_tx.subscribe();
    let pattern_executor_shutdown = shutdown_tx.subscribe();
    let led_mux_shutdown = shutdown_tx.subscribe();
    let led_shutdown = shutdown_tx.subscribe();
    let lcd_shutdown = shutdown_tx.subscribe();
    let temp_led_shutdown = shutdown_tx.subscribe();
    let humidity_led_shutdown = shutdown_tx.subscribe();

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

    // Spawn pattern cycle button task
    let pattern_button_handle = tokio::spawn(async move {
        let mut shutdown = pattern_button_shutdown;
        tokio::select! {
            result = run_pattern_button(pattern_event_tx) => {
                if let Err(e) = result {
                    eprintln!("Pattern button error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Pattern Button] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Pattern cycle button spawned");

    // Spawn DHT11 sensor task
    let dht11_handle = tokio::spawn(async move {
        let shutdown = dht11_shutdown;
        if let Err(e) = run_dht11_sensor(dht11_event_tx, shutdown).await {
            eprintln!("DHT11 sensor error: {}", e);
        }
    });
    println!("  ✓ DHT11 sensor spawned");

    // === CONTROLLER ===

    // Clone command senders so we can use them for shutdown
    let shutdown_lcd_tx = lcd_cmd_tx.clone();

    // Spawn controller task
    let controller_handle = tokio::spawn(async move {
        let mut shutdown = controller_shutdown;
        tokio::select! {
            result = controller::run_controller(event_rx, rgb_led_cmd_tx, lcd_cmd_tx, pattern_tx, temp_led_cmd_tx, humidity_led_cmd_tx) => {
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

    // === PATTERN EXECUTOR ===

    // Spawn pattern executor task
    let pattern_executor_handle = tokio::spawn(async move {
        let mut shutdown = pattern_executor_shutdown;
        tokio::select! {
            result = run_pattern_executor(pattern_rx, pattern_led_tx) => {
                if let Err(e) = result {
                    eprintln!("Pattern executor error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Pattern Executor] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Pattern executor spawned");

    // === LED COMMAND MULTIPLEXER ===

    // Create merged channel for RGB LED (merges controller and pattern commands)
    let (merged_led_tx, merged_led_rx) = mpsc::channel(64);
    let merged_led_tx_clone = merged_led_tx.clone();

    // Spawn multiplexer task to merge controller and pattern commands
    let led_mux_handle = tokio::spawn(async move {
        let mut shutdown = led_mux_shutdown;
        let mut ctrl_rx = rgb_led_cmd_rx;
        let mut pattern_rx = pattern_led_rx;
        let tx = merged_led_tx;

        tokio::select! {
            _ = async {
                loop {
                    tokio::select! {
                        Some(cmd) = ctrl_rx.recv() => {
                            // Controller commands have priority (for overrides)
                            if let Err(e) = tx.send(cmd).await {
                                eprintln!("[LED Mux] Error forwarding controller command: {}", e);
                                break;
                            }
                        }
                        Some(cmd) = pattern_rx.recv() => {
                            // Pattern commands
                            if let Err(e) = tx.send(cmd).await {
                                eprintln!("[LED Mux] Error forwarding pattern command: {}", e);
                                break;
                            }
                        }
                    }
                }
            } => {}
            _ = shutdown.recv() => {
                println!("[LED Mux] Shutdown signal received");
            }
        }
    });
    println!("  ✓ LED command multiplexer spawned");

    // === ACTUATORS ===

    // Clone for shutdown
    let shutdown_led_tx = merged_led_tx_clone.clone();

    // Spawn RGB LED actuator task (receives merged commands from controller and pattern executor)
    let rgb_led_handle = tokio::spawn(async move {
        let mut shutdown = led_shutdown;
        tokio::select! {
            result = run_rgb_led_actuator(merged_led_rx) => {
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

    // Spawn temperature indicator LED actuator task
    let temp_led_handle = tokio::spawn(async move {
        let mut shutdown = temp_led_shutdown;
        tokio::select! {
            result = run_temperature_led_actuator(temp_led_cmd_rx) => {
                if let Err(e) = result {
                    eprintln!("Temperature LED actuator error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Temperature LEDs] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Temperature indicator LEDs spawned");

    // Spawn humidity indicator LED actuator task
    let humidity_led_handle = tokio::spawn(async move {
        let mut shutdown = humidity_led_shutdown;
        tokio::select! {
            result = run_humidity_led_actuator(humidity_led_cmd_rx) => {
                if let Err(e) = result {
                    eprintln!("Humidity LED actuator error: {}", e);
                }
            }
            _ = shutdown.recv() => {
                println!("[Humidity LEDs] Shutdown signal received");
            }
        }
    });
    println!("  ✓ Humidity indicator LEDs spawned");

    println!("\n=== System Ready ===");
    println!("All components running.");
    println!("");
    println!("Controls:");
    println!("  • PIR sensor: Wave hand to trigger motion detection");
    println!("  • DHT11 sensor: Reads temperature and humidity every 2 seconds");
    println!("  • Override button: Toggle Automatic/Manual mode");
    println!("  • Brightness button: Cycle brightness (25% → 50% → 75% → 100%)");
    println!("  • Pattern button: Cycle lighting patterns");
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
        pattern_button_handle,
        dht11_handle,
        controller_handle,
        pattern_executor_handle,
        led_mux_handle,
        rgb_led_handle,
        lcd_handle,
        temp_led_handle,
        humidity_led_handle
    );

    println!("\nGPIO System shut down cleanly.");

    Ok(())
}
