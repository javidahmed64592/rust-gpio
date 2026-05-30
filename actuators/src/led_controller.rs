//! Generic LED Controller Module

use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};

/// Generic LED controller for any GPIO pin
pub struct LedController {
    pin: OutputPin,
    label: String,
}

impl LedController {
    /// Initialize LED controller with specified GPIO pin and label
    pub fn new(pin_number: u8, label: &str) -> Result<Self> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_number)?.into_output();
        println!("[{}] Initialized on GPIO pin: {}", label, pin_number);
        Ok(Self {
            pin,
            label: label.to_string(),
        })
    }

    /// Turn LED on
    pub fn turn_on(&mut self) {
        self.pin.set_high();
        println!("[{}] ON", self.label);
    }

    /// Turn LED off
    pub fn turn_off(&mut self) {
        self.pin.set_low();
        println!("[{}] OFF", self.label);
    }

    /// Set brightness using PWM (0-100)
    /// TODO: Implement proper software PWM for brightness control
    pub fn set_brightness(&mut self, level: u8) {
        if level > 0 {
            self.turn_on();
        } else {
            self.turn_off();
        }
        println!("[{}] Brightness: {}%", self.label, level);
    }
}
