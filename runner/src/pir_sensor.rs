//! PIR Sensor Task Implementation

use anyhow::Result;
use gpio_core::{Event, load_config};
use sensors::PirSensorController;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

/// Run the PIR sensor task with an event sender channel
pub async fn run_pir_sensor(event_tx: mpsc::Sender<Event>) -> Result<()> {
    // Load config to get PIR pin
    let config = load_config("config/config.yaml")?;
    let pir_pin_number = config.gpio.pir.pin;

    // Initialize PIR sensor controller
    let mut pir = PirSensorController::new(pir_pin_number, "PIR")?;

    println!("[PIR] Ready to detect motion!");

    // Poll the PIR sensor
    loop {
        // Check for motion
        if let Some(event) = pir.check_motion() {
            if let Err(e) = event_tx.send(event).await {
                eprintln!("[PIR] Failed to send event: {}", e);
                break;
            }
        }

        // Poll every 100ms to avoid excessive CPU usage
        sleep(Duration::from_millis(100)).await;
    }

    println!("[PIR] Shutting down...");
    Ok(())
}
