//! Core types and shared functionality for the GPIO event system.
//!
//! This crate provides:
//! - Event types (sensor outputs)
//! - Command types (actuator inputs)
//! - System state models
//! - Configuration loading
//! - Shared traits and error types

use serde::{Deserialize, Serialize};

/// Events emitted by sensors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// PIR sensor detected motion
    MotionDetected,
    /// Lighting mode toggle button was pressed
    LightingModeTogglePressed,
    /// Brightness adjustment button was pressed
    BrightnessButtonPressed,
}

/// Commands sent to actuators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Turn LED on at previous brightness
    LedOn,
    /// Turn LED off
    LedOff,
    /// Set LED brightness (0-100)
    SetBrightness(u8),
    /// Blink LED to indicate error (times to blink)
    LedBlinkError(u8),

    /// Display text on specified LCD line
    DisplayText {
        line: u8,
        text: String,
    },
    /// Clear all text from LCD
    ClearDisplay,
    /// Turn LCD backlight on
    DisplayOn,
    /// Turn LCD backlight off
    DisplayOff,
}

/// Lighting control mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingMode {
    /// PIR sensor controls lighting automatically
    Automatic,
    /// Manual override - PIR ignored
    ManualOverride,
}

/// System-wide configuration loaded from config.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// GPIO pin configurations
    pub gpio: GpioConfig,
    /// LCD display configuration
    pub lcd: LcdConfig,
    /// System behavior configuration
    pub system: SystemConfig,
}

/// GPIO hardware pin mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    /// LED pin configuration
    pub led: LedPinConfig,
    /// Button pin configurations
    pub button: ButtonPinConfig,
    /// PIR sensor configuration
    pub pir: PirConfig,
}

/// LED GPIO pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedPinConfig {
    /// GPIO pin number for PIR-controlled LED
    pub pir_led_pin: u8,
}

/// Button GPIO pin configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonPinConfig {
    /// GPIO pin for lighting mode override button
    pub lighting_override_pin: u8,
    /// GPIO pin for brightness adjustment button
    pub lighting_brightness_pin: u8,
}

/// PIR sensor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PirConfig {
    /// GPIO pin number for PIR sensor
    pub pin: u8,
}

/// LCD display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcdConfig {
    /// I2C address for LCD display (e.g., 0x27)
    pub i2c_address: u8,
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Duration in seconds before presence times out
    pub presence_timeout_secs: u64,
    /// Default LED brightness on startup (0-100)
    pub default_brightness: u8,
    /// Configurable brightness levels to cycle through
    pub brightness_levels: Vec<u8>,
}

/// Load configuration from YAML file
///
/// # Arguments
/// * `path` - Path to the config.yaml file
///
/// # Returns
/// Parsed configuration or error if file cannot be read/parsed
pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Global system state maintained by the controller
#[derive(Debug, Clone)]
pub struct SystemState {
    /// Whether presence is currently detected
    pub presence_detected: bool,
    /// Current lighting control mode
    pub lighting_mode: LightingMode,
    /// Current brightness level (0-100)
    pub brightness_level: u8,
    /// Timestamp of last motion detection
    pub last_motion_time: Option<std::time::Instant>,
    /// Index in brightness_levels array
    pub brightness_index: usize,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            presence_detected: false,
            lighting_mode: LightingMode::Automatic,
            brightness_level: 100,
            last_motion_time: None,
            brightness_index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_system_state() {
        let state = SystemState::default();
        assert_eq!(state.lighting_mode, LightingMode::Automatic);
        assert_eq!(state.brightness_level, 80);
        assert!(!state.presence_detected);
    }
}
