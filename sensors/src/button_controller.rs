//! Generic Button Implementation Module

use anyhow::Result;
use gpio_core::Event;
use rppal::gpio::{Gpio, InputPin, Level};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

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

/// Run a generic button with configurable GPIO pin and event emission
///
/// # Arguments
/// * `pin_number` - GPIO pin number the button is connected to
/// * `event` - Event to emit when button is pressed
/// * `label` - Label for logging (e.g., "Brightness", "Override")
/// * `event_tx` - Channel to send events through
pub async fn run_button(
    pin_number: u8,
    event: Event,
    label: &str,
    event_tx: mpsc::Sender<Event>,
) -> Result<()> {
    // Initialize button controller
    let mut button = ButtonController::new(pin_number, label)?;

    println!("[{}] Ready! Press button to emit event.", label);

    // Debounce time
    const DEBOUNCE_MS: u64 = 300;

    loop {
        // Check for button press
        if button.is_pressed() {
            // Send event
            if let Err(e) = event_tx.send(event.clone()).await {
                eprintln!("[{}] Failed to send event: {}", label, e);
                break;
            }

            // Debounce - wait before checking again
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;
        }

        // Poll every 50ms
        sleep(Duration::from_millis(50)).await;
    }

    println!("[{}] Shutting down...", label);
    Ok(())
}
