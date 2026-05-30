//! PIR Sensor Implementation Module

use anyhow::Result;
use gpio_core::{Event, load_config};
use rppal::gpio::{Gpio, InputPin, Level};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

/// PIR (Passive Infrared) motion sensor controller
pub struct PirSensorController {
    pin: InputPin,
    label: String,
    previous_state: Level,
}

impl PirSensorController {
    /// Initialize PIR sensor controller with specified GPIO pin and label
    ///
    /// # Arguments
    /// * `pin_number` - GPIO pin number the PIR sensor is connected to
    /// * `label` - Label for logging (e.g., "PIR", "Motion Sensor")
    pub fn new(pin_number: u8, label: &str) -> Result<Self> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_number)?.into_input();
        let previous_state = pin.read();

        println!("[{}] Initialized on GPIO pin: {}", label, pin_number);

        if previous_state == Level::High {
            println!("[{}] Initial state: MOTION DETECTED", label);
        } else {
            println!("[{}] Initial state: No motion", label);
        }

        Ok(Self {
            pin,
            label: label.to_string(),
            previous_state,
        })
    }

    /// Check for motion detection (LOW -> HIGH transition)
    /// Returns Some(Event) if motion is detected, None otherwise
    pub fn check_motion(&mut self) -> Option<Event> {
        let current_state = self.pin.read();

        // Detect state change
        if current_state != self.previous_state {
            self.previous_state = current_state;

            match current_state {
                Level::High => {
                    // Motion detected (LOW -> HIGH transition)
                    println!("[{}] Motion DETECTED!", self.label);
                    return Some(Event::MotionDetected);
                }
                Level::Low => {
                    // PIR hardware timeout expired (pin went LOW)
                    println!("[{}] PIR sensor pin LOW (hardware timeout)", self.label);
                }
            }
        }

        None
    }
}

/// Run the PIR sensor with an event sender channel
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
