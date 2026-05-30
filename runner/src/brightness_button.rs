//! Brightness Button Task Implementation
//!
//! Spawns an async task to poll the brightness adjustment button.

use anyhow::Result;
use gpio_core::{Event, load_config};
use sensors::ButtonController;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

/// Run the brightness button task with an event sender channel
///
/// # Arguments
/// * `event_tx` - Channel to send button press events to the controller
///
/// # Behavior
/// Polls button every 50ms with 300ms debounce, emits `BrightnessButtonPressed`
pub async fn run_brightness_button(event_tx: mpsc::Sender<Event>) -> Result<()> {
    // Load config to get button pin
    let config = load_config("config/config.yaml")?;
    let pin_number = config.gpio.button.lighting_brightness_pin;

    // Initialize button controller
    let mut button = ButtonController::new(pin_number, "Brightness Button")?;

    println!(
        "[Brightness Button] Ready! Press button to cycle brightness (25% → 50% → 75% → 100%)"
    );

    // Debounce time
    const DEBOUNCE_MS: u64 = 300;

    loop {
        // Check for button press
        if button.is_pressed() {
            // Send event
            if let Err(e) = event_tx.send(Event::BrightnessButtonPressed).await {
                eprintln!("[Brightness Button] Failed to send event: {}", e);
                break;
            }

            // Debounce - wait before checking again
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;
        }

        // Poll every 50ms
        sleep(Duration::from_millis(50)).await;
    }

    println!("[Brightness Button] Shutting down...");
    Ok(())
}
