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
    /// PIR sensor motion timeout expired
    MotionExpired,
    /// Lighting mode toggle button was pressed
    LightingModeTogglePressed,
    /// Brightness adjustment button was pressed
    BrightnessButtonPressed,
    /// MPU6050 tilt update
    TiltUpdated { pitch: f32, roll: f32 },
}

/// Commands sent to actuators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// LED commands
    LedOn,
    LedOff,
    SetBrightness(u8),
    
    /// LCD commands
    DisplayText { line: u8, text: String },
    ClearDisplay,
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
    pub gpio: GpioConfig,
    pub lcd: LcdConfig,
    pub mpu6050: Mpu6050Config,
    pub system: SystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    pub led: LedPinConfig,
    pub button: ButtonPinConfig,
    pub pir: PirConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedPinConfig {
    pub pir_led_pin: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonPinConfig {
    pub lighting_override_pin: u8,
    pub lighting_brightness_pin: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PirConfig {
    pub pin: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcdConfig {
    pub i2c_address: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mpu6050Config {
    pub i2c_bus: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub presence_timeout_secs: u64,
    pub default_brightness: u8,
}

/// Load configuration from YAML file
pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Global system state maintained by the controller
#[derive(Debug, Clone)]
pub struct SystemState {
    pub presence_detected: bool,
    pub lighting_mode: LightingMode,
    pub brightness_level: u8,
    pub last_motion_time: Option<std::time::Instant>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            presence_detected: false,
            lighting_mode: LightingMode::Automatic,
            brightness_level: 80,
            last_motion_time: None,
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
