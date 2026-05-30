//! Generic Button Controller Module

use anyhow::Result;
use rppal::gpio::{Gpio, InputPin, Level};

/// Generic button controller for any GPIO pin
pub struct ButtonController {
    pin: InputPin,
    label: String,
    last_state: Level,
}

impl ButtonController {
    /// Initialize button controller with specified GPIO pin and label
    ///
    /// # Arguments
    /// * `pin_number` - GPIO pin number the button is connected to
    /// * `label` - Label for logging (e.g., "Brightness", "Override")
    pub fn new(pin_number: u8, label: &str) -> Result<Self> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_number)?.into_input_pullup();
        let last_state = pin.read();

        println!("[{}] Initialized on GPIO pin: {}", label, pin_number);

        Ok(Self {
            pin,
            label: label.to_string(),
            last_state,
        })
    }

    /// Check if button was pressed (falling edge: HIGH -> LOW)
    /// Returns true if a press was detected
    pub fn is_pressed(&mut self) -> bool {
        let current_state = self.pin.read();
        let pressed = self.last_state == Level::High && current_state == Level::Low;
        self.last_state = current_state;

        if pressed {
            println!("[{}] Button pressed!", self.label);
        }

        pressed
    }
}
