//! Generic Button Implementation Module

use anyhow::Result;
use gpio_core::Event;
use rppal::gpio::{Gpio, Level};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

/// Run a generic button with configurable GPIO pin and event emission
///
/// # Arguments
/// * `pin_number` - GPIO pin number the button is connected to
/// * `event` - Event to emit when button is pressed
/// * `label` - Optional label for logging (e.g., "Brightness", "Override")
/// * `event_tx` - Channel to send events through
pub async fn run_button(
    pin_number: u8,
    event: Event,
    label: &str,
    event_tx: mpsc::Sender<Event>,
) -> Result<()> {
    println!("[{}] Initializing on GPIO pin: {}", label, pin_number);

    // Initialize GPIO pin for button with pull-up resistor
    let gpio = Gpio::new()?;
    let button_pin = gpio.get(pin_number)?.into_input_pullup();

    println!("[{}] Ready! Press button to emit event.", label);

    // Debounce time and tracking
    const DEBOUNCE_MS: u64 = 300;
    let mut last_state = button_pin.read();

    loop {
        // Poll button state
        let current_state = button_pin.read();

        // Detect button press (falling edge: HIGH -> LOW)
        if last_state == Level::High && current_state == Level::Low {
            println!("[{}] Button pressed!", label);

            // Send event
            if let Err(e) = event_tx.send(event.clone()).await {
                eprintln!("[{}] Failed to send event: {}", label, e);
                break;
            }

            // Debounce - wait and update state
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;
        }

        last_state = current_state;

        // Poll every 50ms
        sleep(Duration::from_millis(50)).await;
    }

    println!("[{}] Shutting down...", label);
    Ok(())
}
