//! Lighting Pattern Cycle Button Task Implementation

use anyhow::Result;
use gpio_core::{Event, load_config};
use sensors::ButtonController;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

/// Run the lighting pattern cycle button task
///
/// # Arguments
/// * `event_tx` - Channel to send PatternCyclePressed events
///
/// # Behavior
/// Polls button every 50ms with 300ms debounce, emits `PatternCyclePressed`
pub async fn run_pattern_button(event_tx: mpsc::Sender<Event>) -> Result<()> {
    // Load config to get pattern button pin
    let config = load_config("config/config.yaml")?;
    let button_pin = config.gpio.button.lighting_pattern_pin;

    // Initialize button controller
    let mut button = ButtonController::new(button_pin, "Pattern Cycle")?;

    println!("[Pattern Button] Ready! Press button to cycle lighting patterns");

    // Debounce time
    const DEBOUNCE_MS: u64 = 300;

    loop {
        // Check for button press
        if button.is_pressed() {
            // Send event
            if let Err(e) = event_tx.send(Event::PatternCyclePressed).await {
                eprintln!("[Pattern Button] Failed to send event: {}", e);
                break;
            }

            // Debounce - wait before checking again
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;
        }

        // Poll every 50ms
        sleep(Duration::from_millis(50)).await;
    }

    println!("[Pattern Button] Shutting down...");
    Ok(())
}
