//! LCD Display Actuator
//!
//! Controls the I2C LCD display.
//! Consumes commands: DisplayText, ClearDisplay.
//! Contains no business logic - purely renders what it's told.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("LCD Actuator starting...");

    // TODO: Load config
    // TODO: Initialize I2C bus
    // TODO: Initialize LCD device
    // TODO: Set up command channel
    // TODO: Listen for commands (DisplayText, ClearDisplay)
    // TODO: Render text without embedded logic

    println!("LCD Actuator initialized (placeholder)");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("LCD Actuator shutting down...");

    Ok(())
}
