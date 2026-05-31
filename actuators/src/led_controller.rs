//! Generic LED Controller Module

use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

/// Generic LED controller for any GPIO pin with PWM support
pub struct LedController {
    pin: OutputPin,
    label: String,
    current_brightness: u8,
    previous_brightness: u8,
}

impl LedController {
    /// Initialize LED controller with specified GPIO pin and label
    ///
    /// # Arguments
    /// * `pin_number` - GPIO pin number the LED is connected to
    /// * `label` - Label for logging (e.g., "PIR LED", "Status LED")
    ///
    /// # Returns
    /// New LED controller instance or error if GPIO initialization fails
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

    /// Turn LED off (preserves previous brightness for later restore)
    pub fn turn_off(&mut self) {
        self.set_brightness(0);
    }

    /// Set brightness using PWM (0-100)
    ///
    /// # Arguments
    /// * `level` - Brightness percentage (0-100, clamped automatically)
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

    /// Blink LED rapidly to indicate an error condition
    ///
    /// # Arguments
    /// * `times` - Number of times to blink (blocking operation)
    pub fn blink_error(&mut self, times: u8) {
        let original_brightness = self.current_brightness;

        for i in 0..times {
            // Fast blink pattern: 100ms on, 100ms off
            self.set_brightness(100);
            thread::sleep(Duration::from_millis(100));
            self.set_brightness(0);

            // Short pause between blinks except on last one
            if i < times - 1 {
                thread::sleep(Duration::from_millis(100));
            }
        }

        // Restore original brightness after blinking
        thread::sleep(Duration::from_millis(200));
        self.set_brightness(original_brightness);
        println!("[{}] Error blink pattern completed", self.label);
    }

    /// Get current brightness level (0-100)
    pub fn get_brightness(&self) -> u8 {
        self.current_brightness
    }
}
