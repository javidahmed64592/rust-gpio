//! PIR Sensor Controller Module

use anyhow::Result;
use gpio_core::Event;
use rppal::gpio::{Gpio, InputPin, Level};

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
