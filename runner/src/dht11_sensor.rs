//! DHT11 Sensor Task Implementation

use anyhow::Result;
use gpio_core::{Event, load_config};
use sensors::Dht11Controller;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, interval};

/// Run the DHT11 sensor task with event sender channel
///
/// # Arguments
/// * `event_tx` - Channel to send environment reading events to the controller
/// * `mut shutdown_rx` - Shutdown signal receiver
///
/// # Behavior
/// Polls DHT11 sensor every 2 seconds and emits EnvironmentReading events.
/// DHT11 requires at least 1 second between reads for stability.
/// Respects shutdown signal to terminate gracefully.
pub async fn run_dht11_sensor(
    event_tx: mpsc::Sender<Event>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    // Load config to get DHT11 pin
    let config = load_config("config/config.yaml")?;
    let dht11_pin = config.gpio.dht11.pin;

    // Initialize DHT11 controller
    let mut dht11 = Dht11Controller::new(dht11_pin, "DHT11")?;

    println!("[DHT11 Sensor] Ready to read temperature and humidity!");

    // Poll sensor every 2 seconds (DHT11 needs at least 1 second between reads)
    let mut poll_interval = interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = poll_interval.tick() => {
                // Read temperature and humidity (now async)
                if let Some(event) = dht11.read_environment().await {
                    // Send event to controller
                    if let Err(e) = event_tx.send(event).await {
                        eprintln!("[DHT11 Sensor] Failed to send event: {}", e);
                        break;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                println!("[DHT11 Sensor] Shutdown signal received");
                break;
            }
        }
    }

    println!("[DHT11 Sensor] Shutting down...");
    Ok(())
}
