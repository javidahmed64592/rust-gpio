//! Controller library
//!
//! Central decision-making component for the GPIO system.

pub mod controller_impl;

// Re-export the main run function
pub use controller_impl::run_controller;
