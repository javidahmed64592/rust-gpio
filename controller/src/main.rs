//! Controller - The Brain of the GPIO System
//!
//! Central decision-making component that:
//! - Receives events from all sensors
//! - Maintains global SystemState
//! - Applies all business logic
//! - Generates commands for actuators
//!
//! This is the ONLY place where behavioral logic should exist.

use anyhow::Result;
use gpio_core::{Command, Event, SystemState};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Controller starting...");

    // TODO: Load config
    // TODO: Initialize SystemState
    // TODO: Set up event receiver channel
    // TODO: Set up command sender channels (LED, LCD)
    // TODO: Implement main event loop
    // TODO: Handle MotionDetected/MotionExpired
    // TODO: Handle LightingModeTogglePressed
    // TODO: Handle BrightnessButtonPressed
    // TODO: Handle TiltUpdated
    // TODO: Implement presence timeout logic
    // TODO: Apply lighting mode rules
    // TODO: Generate LED commands
    // TODO: Generate LCD commands

    let mut state = SystemState::default();
    println!("Controller initialized with state: {:?}", state);
    println!("Initial lighting mode: {:?}", state.lighting_mode);
    println!("Initial brightness: {}%", state.brightness_level);

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("Controller shutting down...");

    Ok(())
}

/// Process a sensor event and update system state
fn handle_event(event: Event, state: &mut SystemState) -> Vec<Command> {
    // TODO: Implement event handling logic
    // This is where all behavioral rules go
    vec![]
}
