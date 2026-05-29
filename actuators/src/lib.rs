//! Actuators library
//!
//! Provides actuator functions that can be spawned as tasks.

mod led_impl;

// Re-export the main run function
pub use led_impl::run_led_actuator;
