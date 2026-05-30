//! Sensors library
//!
//! Provides sensor functions that can be spawned as tasks.

mod brightness_impl;
mod button_controller;
mod lighting_override_impl;
mod pir_impl;

// Re-export the main run functions and button controller
pub use brightness_impl::run_brightness_button;
pub use button_controller::{ButtonController, run_button};
pub use lighting_override_impl::run_lighting_override_button;
pub use pir_impl::run_pir_sensor;
