//! Generic LED Controller Module

use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};

/// Generic LED controller for any GPIO pin
pub struct LedController {
    pin: OutputPin,
    label: String,
    current_brightness: u8,
    previous_brightness: u8,
}

impl LedController {
    /// Initialize LED controller with specified GPIO pin and label
    pub fn new(pin_number: u8, label: &str) -> Result<Self> {
        let gpio = Gpio::new()?;
        let mut pin = gpio.get(pin_number)?.into_output();

        // Set up software PWM at 100Hz frequency
        // This is a good balance between smoothness and CPU usage
        pin.set_pwm_frequency(100.0, 0.0)?;

        println!(
            "[{}] Initialized on GPIO pin: {} with PWM support",
            label, pin_number
        );
        Ok(Self {
            pin,
            label: label.to_string(),
            current_brightness: 0,
            previous_brightness: 100, // Default to 100% when first turned on
        })
    }

    /// Turn LED on (restores previous brightness level)
    pub fn turn_on(&mut self) {
        self.set_brightness(self.previous_brightness);
    }

    /// Turn LED off
    pub fn turn_off(&mut self) {
        self.set_brightness(0);
    }

    /// Set brightness using PWM (0-100)
    pub fn set_brightness(&mut self, level: u8) {
        let level = level.min(100); // Clamp to 0-100 range

        // Remember non-zero brightness for when we turn back on
        if level > 0 {
            self.previous_brightness = level;
        }

        self.current_brightness = level;

        // Convert percentage (0-100) to duty cycle (0.0-1.0)
        let duty_cycle = level as f64 / 100.0;

        // Set PWM duty cycle
        if let Err(e) = self.pin.set_pwm_frequency(100.0, duty_cycle) {
            eprintln!("[{}] Failed to set PWM: {}", self.label, e);
        }

        println!("[{}] Brightness: {}%", self.label, level);
    }

    /// Get current brightness level (0-100)
    pub fn get_brightness(&self) -> u8 {
        self.current_brightness
    }
}
