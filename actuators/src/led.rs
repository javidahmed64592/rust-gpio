//! LED Actuator
//!
//! Controls desk lighting via LED(s).
//! Consumes commands: LedOn, LedOff, SetBrightness.
//! Contains no business logic - purely command-driven.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("LED Actuator starting...");

    // TODO: Load config
    // TODO: Initialize GPIO pin for LED
    // TODO: Set up command channel
    // TODO: Listen for commands (LedOn, LedOff, SetBrightness)
    // TODO: Apply PWM for brightness control
    // TODO: Execute commands without embedded logic

    println!("LED Actuator initialized (placeholder)");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("LED Actuator shutting down...");

    Ok(())
}
