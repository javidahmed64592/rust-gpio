//! Sensors library
//!
//! Provides sensor functions that can be spawned as tasks.

mod pir_impl;

// Re-export the main run functions
pub use pir_impl::run_pir_sensor;
