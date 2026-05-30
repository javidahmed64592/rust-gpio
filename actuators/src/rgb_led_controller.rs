//! RGB LED Controller Module

use anyhow::Result;
use gpio_core::RgbColor;
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

/// RGB LED controller with independent PWM control for each color channel
pub struct RgbLedController {
    red_pin: OutputPin,
    green_pin: OutputPin,
    blue_pin: OutputPin,
    label: String,
    current_color: RgbColor,
    current_brightness: u8,
}

impl RgbLedController {
    /// Initialize RGB LED controller with specified GPIO pins and label
    ///
    /// # Arguments
    /// * `red_pin_number` - GPIO pin number for red channel
    /// * `green_pin_number` - GPIO pin number for green channel
    /// * `blue_pin_number` - GPIO pin number for blue channel
    /// * `label` - Label for logging (e.g., "System RGB LED")
    ///
    /// # Returns
    /// New RGB LED controller instance or error if GPIO initialization fails
    pub fn new(
        red_pin_number: u8,
        green_pin_number: u8,
        blue_pin_number: u8,
        label: &str,
    ) -> Result<Self> {
        let gpio = Gpio::new()?;

        // Initialize all three color channels with PWM at 100Hz
        let mut red_pin = gpio.get(red_pin_number)?.into_output();
        let mut green_pin = gpio.get(green_pin_number)?.into_output();
        let mut blue_pin = gpio.get(blue_pin_number)?.into_output();

        red_pin.set_pwm_frequency(100.0, 0.0)?;
        green_pin.set_pwm_frequency(100.0, 0.0)?;
        blue_pin.set_pwm_frequency(100.0, 0.0)?;

        println!(
            "[{}] Initialized on GPIO pins - R:{} G:{} B:{} with PWM support",
            label, red_pin_number, green_pin_number, blue_pin_number
        );

        Ok(Self {
            red_pin,
            green_pin,
            blue_pin,
            label: label.to_string(),
            current_color: RgbColor::off(),
            current_brightness: 100,
        })
    }

    /// Turn RGB LED on with the last set color at current brightness
    pub fn turn_on(&mut self) {
        self.set_color(self.current_color);
    }

    /// Turn RGB LED off (all channels to 0)
    pub fn turn_off(&mut self) {
        self.set_color(RgbColor::off());
    }

    /// Set RGB LED color with current brightness applied
    ///
    /// # Arguments
    /// * `color` - RGB color to display (will be scaled by current brightness)
    pub fn set_color(&mut self, color: RgbColor) {
        // Store the base color (without brightness applied)
        self.current_color = color;

        // Apply brightness scaling
        let scaled_color = color.with_brightness(self.current_brightness);

        // Set PWM duty cycle for each channel (0-100 -> 0.0-1.0)
        let red_duty = scaled_color.red as f64 / 100.0;
        let green_duty = scaled_color.green as f64 / 100.0;
        let blue_duty = scaled_color.blue as f64 / 100.0;

        if let Err(e) = self.red_pin.set_pwm_frequency(100.0, red_duty) {
            eprintln!("[{}] Failed to set red PWM: {}", self.label, e);
        }
        if let Err(e) = self.green_pin.set_pwm_frequency(100.0, green_duty) {
            eprintln!("[{}] Failed to set green PWM: {}", self.label, e);
        }
        if let Err(e) = self.blue_pin.set_pwm_frequency(100.0, blue_duty) {
            eprintln!("[{}] Failed to set blue PWM: {}", self.label, e);
        }
    }

    /// Set overall brightness level (0-100) without changing color
    ///
    /// # Arguments
    /// * `level` - Brightness percentage (0-100, clamped automatically)
    pub fn set_brightness(&mut self, level: u8) {
        self.current_brightness = level.min(100);

        // Re-apply current color with new brightness
        self.set_color(self.current_color);
    }

    /// Blink LED rapidly in red to indicate an error condition
    ///
    /// # Arguments
    /// * `times` - Number of times to blink (blocking operation)
    pub fn blink_error(&mut self, times: u8) {
        let original_color = self.current_color;
        let original_brightness = self.current_brightness;

        for i in 0..times {
            // Fast blink pattern: red at full brightness
            self.set_brightness(100);
            self.set_color(RgbColor::red());
            thread::sleep(Duration::from_millis(100));
            self.set_color(RgbColor::off());

            // Short pause between blinks except on last one
            if i < times - 1 {
                thread::sleep(Duration::from_millis(100));
            }
        }

        // Restore original color and brightness after blinking
        thread::sleep(Duration::from_millis(200));
        self.set_brightness(original_brightness);
        self.set_color(original_color);
        println!("[{}] Error blink pattern completed", self.label);
    }

    /// Get current RGB color (base color without brightness applied)
    pub fn get_color(&self) -> RgbColor {
        self.current_color
    }

    /// Get current brightness level (0-100)
    pub fn get_brightness(&self) -> u8 {
        self.current_brightness
    }
}
