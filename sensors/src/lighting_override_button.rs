//! Lighting Override Button
//!
//! Toggles between Automatic and Manual Override lighting modes.
//! Emits LightingModeTogglePressed events.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Lighting Override Button starting...");

    // TODO: Load config
    // TODO: Initialize GPIO pin for button
    // TODO: Set up event channel
    // TODO: Implement debounce logic
    // TODO: Listen for button presses
    // TODO: Emit LightingModeTogglePressed events

    println!("Lighting Override Button initialized (placeholder)");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("Lighting Override Button shutting down...");

    Ok(())
}
