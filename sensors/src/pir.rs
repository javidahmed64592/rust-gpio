//! PIR (Passive Infrared) Sensor
//!
//! Detects motion/presence at the desk.
//! Emits MotionDetected and MotionExpired events.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("PIR Sensor starting...");

    // TODO: Load config
    // TODO: Initialize GPIO pin for PIR sensor
    // TODO: Set up event channel
    // TODO: Poll PIR sensor state
    // TODO: Emit MotionDetected/MotionExpired events

    println!("PIR Sensor initialized (placeholder)");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("PIR Sensor shutting down...");

    Ok(())
}
