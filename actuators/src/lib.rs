//! Actuators library
//!
//! Provides actuator functions that can be spawned as tasks.

mod led_controller;
mod pir_led_impl;

// Re-export the main run function and LED controller
pub use led_controller::LedController;
pub use pir_led_impl::run_pir_led_actuator;
