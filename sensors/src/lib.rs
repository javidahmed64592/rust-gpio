//! Sensors library
//!
//! Provides sensor functions that can be spawned as tasks.

mod brightness_impl;
mod button_impl;
mod lighting_override_impl;
mod pir_impl;

// Re-export the main run functions
pub use brightness_impl::run_brightness_button;
pub use button_impl::run_button;
pub use lighting_override_impl::run_lighting_override_button;
pub use pir_impl::run_pir_sensor;
