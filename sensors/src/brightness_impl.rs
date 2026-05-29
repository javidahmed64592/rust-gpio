//! Brightness Button Implementation Module

use anyhow::Result;
use gpio_core::{Event, load_config};
use tokio::sync::mpsc;

use crate::button_impl::run_button;

/// Run the brightness button with an event sender channel
pub async fn run_brightness_button(event_tx: mpsc::Sender<Event>) -> Result<()> {
    let config = load_config("config/config.yaml")?;
    let pin_number = config.gpio.button.lighting_brightness_pin;

    run_button(
        pin_number,
        Event::BrightnessButtonPressed,
        "Brightness Button",
        event_tx,
    )
    .await
}
