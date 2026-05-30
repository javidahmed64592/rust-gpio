//! Core types and shared functionality for the GPIO event system.
//!
//! This crate provides:
//! - Event types (sensor outputs)
//! - Command types (actuator inputs)
//! - System state models
//! - Configuration loading
//! - Shared traits and error types

use serde::{Deserialize, Serialize};

/// RGB color representation with 0-100 intensity for each channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    /// Create a new RGB color with specified intensities (0-100)
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red.min(100),
            green: green.min(100),
            blue: blue.min(100),
        }
    }

    /// Predefined color: Red (for errors)
    pub fn red() -> Self {
        Self::new(100, 0, 0)
    }

    /// Predefined color: Green (for normal operation)
    pub fn green() -> Self {
        Self::new(0, 100, 0)
    }

    /// Predefined color: Blue (for motion detected)
    pub fn blue() -> Self {
        Self::new(0, 0, 100)
    }

    /// Predefined color: Orange (for system busy)
    pub fn orange() -> Self {
        Self::new(100, 50, 0)
    }

    /// Predefined color: Off (all channels at 0)
    pub fn off() -> Self {
        Self::new(0, 0, 0)
    }

    /// Scale all color channels by a brightness percentage (0-100)
    pub fn with_brightness(&self, brightness: u8) -> Self {
        let brightness = brightness.min(100);
        let scale = brightness as f64 / 100.0;

        Self::new(
            (self.red as f64 * scale) as u8,
            (self.green as f64 * scale) as u8,
            (self.blue as f64 * scale) as u8,
        )
    }

    /// Interpolate between two colors
    ///
    /// # Arguments
    /// * `other` - Target color to interpolate towards
    /// * `t` - Interpolation factor (0.0 = self, 1.0 = other)
    pub fn lerp(&self, other: &RgbColor, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            (self.red as f64 + (other.red as f64 - self.red as f64) * t) as u8,
            (self.green as f64 + (other.green as f64 - self.green as f64) * t) as u8,
            (self.blue as f64 + (other.blue as f64 - self.blue as f64) * t) as u8,
        )
    }

    /// Create a color from HSV (Hue 0-360, Saturation 0-100, Value 0-100)
    pub fn from_hsv(hue: f64, saturation: u8, value: u8) -> Self {
        let s = (saturation.min(100) as f64) / 100.0;
        let v = (value.min(100) as f64) / 100.0;
        let h = hue % 360.0;

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as i32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(
            ((r + m) * 100.0) as u8,
            ((g + m) * 100.0) as u8,
            ((b + m) * 100.0) as u8,
        )
    }
}

/// Events emitted by sensors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// PIR sensor detected motion
    MotionDetected,
    /// Lighting mode toggle button was pressed
    LightingModeTogglePressed,
    /// Brightness adjustment button was pressed
    BrightnessButtonPressed,
    /// Lighting pattern cycle button was pressed
    PatternCyclePressed,
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

    /// Turn RGB LED on at previous color and brightness
    RgbLedOn,
    /// Turn RGB LED off
    RgbLedOff,
    /// Set RGB LED color (current brightness applies)
    SetRgbColor(RgbColor),
    /// Set RGB LED brightness (0-100)
    SetRgbBrightness(u8),
    /// Blink RGB LED in red to indicate error
    RgbLedBlinkError(u8),
    /// Set active lighting pattern with optional override flag
    SetLightingPattern {
        pattern_index: usize,
        is_override: bool,
    },

    /// Display text on specified LCD line
    DisplayText { line: u8, text: String },
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
    /// Lighting pattern configurations
    pub lighting_patterns: Vec<LightingPatternConfig>,
}

/// GPIO hardware pin mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    /// LED pin configuration
    pub led: LedPinConfig,
    /// RGB LED pin configuration
    pub rgb_led: RgbLedPinConfig,
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

/// RGB LED GPIO pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbLedPinConfig {
    /// Nested system RGB LED configuration
    pub system: SystemRgbLedPins,
}

/// System RGB LED pin assignments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRgbLedPins {
    /// GPIO pin for red channel
    pub red_pin: u8,
    /// GPIO pin for green channel
    pub green_pin: u8,
    /// GPIO pin for blue channel
    pub blue_pin: u8,
}

/// Button GPIO pin configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonPinConfig {
    /// GPIO pin for lighting mode override button
    pub lighting_override_pin: u8,
    /// GPIO pin for brightness adjustment button
    pub lighting_brightness_pin: u8,
    /// GPIO pin for lighting pattern cycle button
    pub lighting_pattern_pin: u8,
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
    /// Default lighting pattern index
    pub default_pattern_index: usize,
}

/// Lighting pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LightingPatternConfig {
    /// Static single color
    Static { color: RgbColor },
    /// Gradient alternating between two colors
    Gradient {
        color1: RgbColor,
        color2: RgbColor,
        interval_secs: u64,
    },
    /// Rainbow cycle through spectrum
    Rainbow { cycle_secs: u64, steps: usize },
    /// Pulse/breathing effect with single color
    Pulse {
        color: RgbColor,
        period_secs: u64,
        min_brightness: u8,
        max_brightness: u8,
    },
    /// Cycle through list of colors
    ColorCycle {
        colors: Vec<RgbColor>,
        interval_secs: u64,
    },
    /// Event-driven (responds to motion/presence)
    EventDriven,
}

impl LightingPatternConfig {
    /// Get a human-readable name for this pattern
    pub fn name(&self) -> String {
        match self {
            LightingPatternConfig::Static { color } => {
                format!(
                    "Static (R:{} G:{} B:{})",
                    color.red, color.green, color.blue
                )
            }
            LightingPatternConfig::Gradient {
                color1,
                color2,
                interval_secs,
            } => {
                format!(
                    "Gradient (R:{},G:{},B:{} ↔ R:{},G:{},B:{} / {}s)",
                    color1.red,
                    color1.green,
                    color1.blue,
                    color2.red,
                    color2.green,
                    color2.blue,
                    interval_secs
                )
            }
            LightingPatternConfig::Rainbow { cycle_secs, .. } => {
                format!("Rainbow ({}s cycle)", cycle_secs)
            }
            LightingPatternConfig::Pulse {
                color,
                period_secs,
                min_brightness,
                max_brightness,
            } => {
                format!(
                    "Pulse (R:{},G:{},B:{} / {}s / {}%-{}%)",
                    color.red, color.green, color.blue, period_secs, min_brightness, max_brightness
                )
            }
            LightingPatternConfig::ColorCycle {
                colors,
                interval_secs,
            } => {
                format!("ColorCycle ({} colors / {}s)", colors.len(), interval_secs)
            }
            LightingPatternConfig::EventDriven => "EventDriven (Motion Responsive)".to_string(),
        }
    }
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
    /// Current active lighting pattern index
    pub current_pattern_index: usize,
    /// Whether pattern is temporarily overridden (e.g., by error)
    pub pattern_override_active: bool,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            presence_detected: false,
            lighting_mode: LightingMode::Automatic,
            brightness_level: 100,
            last_motion_time: None,
            brightness_index: 0,
            current_pattern_index: 0,
            pattern_override_active: false,
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
