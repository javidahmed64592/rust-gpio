//! Actuators library
//!
//! Provides generic hardware controller abstractions for output devices.

mod lcd_controller;
mod led_controller;
mod rgb_led_controller;

// Re-export hardware controllers
pub use lcd_controller::LcdController;
pub use led_controller::LedController;
pub use rgb_led_controller::RgbLedController;
