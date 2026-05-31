//! Indicator LED Controller for 3-level status display
//!
//! Controls a set of 3 LEDs to indicate low/medium/high levels
//! (e.g., temperature or humidity ranges) with PWM brightness control.

use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};

/// Three-level indicator LED controller (low, medium, high) with PWM brightness
pub struct IndicatorLedController {
    low_pin: OutputPin,
    medium_pin: OutputPin,
    high_pin: OutputPin,
    // PWM for brightness control (we'll use software PWM via duty cycle simulation)
    current_brightness: u8,
    label: String,
}

impl IndicatorLedController {
    /// Initialize indicator LED controller with 3 GPIO pins
    ///
    /// # Arguments
    /// * `low_pin_number` - GPIO pin for low-level indicator
    /// * `medium_pin_number` - GPIO pin for medium-level indicator
    /// * `high_pin_number` - GPIO pin for high-level indicator
    /// * `label` - Label for logging (e.g., "Temperature", "Humidity")
    pub fn new(
        low_pin_number: u8,
        medium_pin_number: u8,
        high_pin_number: u8,
        label: &str,
    ) -> Result<Self> {
        let gpio = Gpio::new()?;

        let mut low_pin = gpio.get(low_pin_number)?.into_output();
        let mut medium_pin = gpio.get(medium_pin_number)?.into_output();
        let mut high_pin = gpio.get(high_pin_number)?.into_output();

        // Initialize PWM at 100Hz with 0% duty cycle (off)
        low_pin.set_pwm_frequency(100.0, 0.0)?;
        medium_pin.set_pwm_frequency(100.0, 0.0)?;
        high_pin.set_pwm_frequency(100.0, 0.0)?;

        println!(
            "[{} Indicators] Initialized - Low: GPIO{}, Medium: GPIO{}, High: GPIO{}",
            label, low_pin_number, medium_pin_number, high_pin_number
        );

        let mut controller = Self {
            low_pin,
            medium_pin,
            high_pin,
            current_brightness: 100,
            label: label.to_string(),
        };

        // Ensure all LEDs start off
        controller.set_all_off();

        Ok(controller)
    }

    /// Set indicator LED states with brightness control
    ///
    /// # Arguments
    /// * `low` - Turn on low-level LED
    /// * `medium` - Turn on medium-level LED
    /// * `high` - Turn on high-level LED
    /// * `brightness` - Brightness level 0-100 (0 = off, 100 = full brightness)
    pub fn set_indicators(&mut self, low: bool, medium: bool, high: bool, brightness: u8) {
        self.current_brightness = brightness.min(100);

        // Calculate duty cycle for PWM (0-100 brightness -> 0.0-1.0 duty cycle)
        let duty_cycle = if brightness > 0 {
            (brightness as f64) / 100.0
        } else {
            0.0
        };

        // Set PWM for each LED based on state and brightness
        let low_duty = if low { duty_cycle } else { 0.0 };
        let medium_duty = if medium { duty_cycle } else { 0.0 };
        let high_duty = if high { duty_cycle } else { 0.0 };

        if let Err(e) = self.low_pin.set_pwm_frequency(100.0, low_duty) {
            eprintln!("[{}] Failed to set low LED PWM: {}", self.label, e);
        }
        if let Err(e) = self.medium_pin.set_pwm_frequency(100.0, medium_duty) {
            eprintln!("[{}] Failed to set medium LED PWM: {}", self.label, e);
        }
        if let Err(e) = self.high_pin.set_pwm_frequency(100.0, high_duty) {
            eprintln!("[{}] Failed to set high LED PWM: {}", self.label, e);
        }

        let status = if brightness == 0 {
            "OFF (brightness=0)"
        } else {
            match (low, medium, high) {
                (true, false, false) => "LOW",
                (false, true, false) => "MEDIUM",
                (false, false, true) => "HIGH",
                (false, false, false) => "OFF",
                _ => "MULTIPLE",
            }
        };

        println!(
            "[{} Indicators] Set to: {} (brightness: {}%)",
            self.label, status, brightness
        );
    }

    /// Turn all indicator LEDs off
    pub fn set_all_off(&mut self) {
        self.set_indicators(false, false, false, 0);
    }

    /// Turn all indicator LEDs on (for testing)
    pub fn set_all_on(&mut self) {
        self.set_indicators(true, true, true, 100);
    }
}

impl Drop for IndicatorLedController {
    fn drop(&mut self) {
        // Clean shutdown - turn all LEDs off
        self.set_all_off();
    }
}
