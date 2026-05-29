//! PIR Sensor Implementation Module

use anyhow::Result;
use gpio_core::{Event, load_config};
use rppal::gpio::{Gpio, Level};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

/// Run the PIR sensor with an event sender channel
pub async fn run_pir_sensor(event_tx: mpsc::Sender<Event>) -> Result<()> {
    // Load config to get PIR pin
    let config = load_config("config/config.yaml")?;
    let pir_pin_number = config.gpio.pir.pin;

    println!("[PIR] Initializing on GPIO pin: {}", pir_pin_number);

    // Initialize GPIO pin for PIR sensor
    let gpio = Gpio::new()?;
    let pir_pin = gpio.get(pir_pin_number)?.into_input();

    println!("[PIR] Ready to detect motion!");

    // Track previous state to detect changes
    let mut previous_state = pir_pin.read();

    if previous_state == Level::High {
        println!("[PIR] Initial state: MOTION DETECTED");
    } else {
        println!("[PIR] Initial state: No motion");
    }

    // Poll the PIR sensor
    loop {
        // Read current state
        let current_state = pir_pin.read();

        // Detect state change
        if current_state != previous_state {
            match current_state {
                Level::High => {
                    // Motion detected (LOW -> HIGH transition)
                    println!("[PIR] Motion DETECTED!");

                    if let Err(e) = event_tx.send(Event::MotionDetected).await {
                        eprintln!("[PIR] Failed to send MotionDetected event: {}", e);
                        break;
                    }
                }
                Level::Low => {
                    // PIR hardware timeout expired (pin went LOW)
                    // Don't send MotionExpired - controller handles presence timeout
                    println!("[PIR] PIR sensor pin LOW (hardware timeout)");
                }
            }

            previous_state = current_state;
        }

        // Poll every 100ms to avoid excessive CPU usage
        sleep(Duration::from_millis(100)).await;
    }

    println!("[PIR] Shutting down...");
    Ok(())
}
