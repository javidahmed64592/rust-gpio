//! Sensors library
//!
//! Provides generic hardware controller abstractions for input devices.

mod button_controller;
mod pir_sensor_controller;

// Re-export hardware controllers
pub use button_controller::ButtonController;
pub use pir_sensor_controller::PirSensorController;
