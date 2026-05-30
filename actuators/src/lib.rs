//! Actuators library
//!
//! Provides generic hardware controller abstractions for output devices.

mod led_controller;
mod lcd_controller;

// Re-export hardware controllers
pub use led_controller::LedController;
pub use lcd_controller::LcdController;
