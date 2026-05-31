//! Lighting Pattern Executor
//!
//! Executes animated lighting patterns and sends color updates to the RGB LED.

use anyhow::Result;
use gpio_core::{Command, LightingPatternConfig, RgbColor, load_config};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

/// Pattern executor state
pub struct PatternExecutor {
    /// Current pattern being executed
    current_pattern: LightingPatternConfig,
    /// Pattern start time (for animations)
    pattern_start_time: Instant,
    /// Channel to send LED commands
    led_tx: mpsc::Sender<Command>,
    /// Current brightness level
    brightness: u8,
}

impl PatternExecutor {
    /// Create a new pattern executor
    pub fn new(
        initial_pattern: LightingPatternConfig,
        led_tx: mpsc::Sender<Command>,
        brightness: u8,
    ) -> Self {
        Self {
            current_pattern: initial_pattern,
            pattern_start_time: Instant::now(),
            led_tx,
            brightness,
        }
    }

    /// Set a new pattern (resets animation timer)
    pub fn set_pattern(&mut self, pattern: LightingPatternConfig) {
        self.current_pattern = pattern;
        self.pattern_start_time = Instant::now();
    }

    /// Update brightness level
    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness;
    }

    /// Calculate current color based on active pattern and elapsed time
    pub fn current_color(&self) -> RgbColor {
        let elapsed = self.pattern_start_time.elapsed();

        match &self.current_pattern {
            LightingPatternConfig::Static { color } => *color,

            LightingPatternConfig::Gradient {
                color1,
                color2,
                interval_secs,
            } => {
                let interval_ms = interval_secs * 1000;
                let cycle_position = (elapsed.as_millis() as u64 % (interval_ms * 2)) as f64;
                let half_interval = interval_ms as f64;

                // Fade from color1 to color2, then back to color1
                if cycle_position < half_interval {
                    let t = cycle_position / half_interval;
                    color1.lerp(color2, t)
                } else {
                    let t = (cycle_position - half_interval) / half_interval;
                    color2.lerp(color1, t)
                }
            }

            LightingPatternConfig::Rainbow {
                cycle_secs,
                steps: _,
            } => {
                let cycle_ms = cycle_secs * 1000;
                let position = (elapsed.as_millis() as u64 % cycle_ms) as f64 / cycle_ms as f64;
                let hue = position * 360.0;
                RgbColor::from_hsv(hue, 100, 100)
            }

            LightingPatternConfig::Pulse {
                color,
                period_secs,
                min_brightness,
                max_brightness,
            } => {
                let period_ms = period_secs * 1000;
                let cycle_position = (elapsed.as_millis() as u64 % period_ms) as f64;
                let half_period = period_ms as f64 / 2.0;

                // Calculate brightness oscillation
                let brightness_range = (*max_brightness as f64) - (*min_brightness as f64);
                let current_brightness = if cycle_position < half_period {
                    // Fade from min to max
                    let t = cycle_position / half_period;
                    *min_brightness as f64 + brightness_range * t
                } else {
                    // Fade from max to min
                    let t = (cycle_position - half_period) / half_period;
                    *max_brightness as f64 - brightness_range * t
                };

                color.with_brightness(current_brightness as u8)
            }

            LightingPatternConfig::ColorCycle {
                colors,
                interval_secs,
            } => {
                if colors.is_empty() {
                    return RgbColor::off();
                }

                let interval_ms = interval_secs * 1000;
                let total_cycle = interval_ms * colors.len() as u64;
                let position = elapsed.as_millis() as u64 % total_cycle;
                let index = (position / interval_ms) as usize;

                colors[index.min(colors.len() - 1)]
            }

            LightingPatternConfig::EventDriven => {
                // EventDriven patterns are handled by the controller, not the executor
                // Pattern executor does not send periodic updates in this mode
                RgbColor::off()
            }
        }
    }

    /// Send color update to LED
    async fn update_led(&self) {
        let color = self.current_color().with_brightness(self.brightness);

        if let Err(e) = self.led_tx.send(Command::SetRgbColor(color)).await {
            eprintln!("[Pattern Executor] Failed to send color update: {}", e);
        }
    }
}

/// Run the pattern executor task
///
/// # Arguments
/// * `pattern_rx` - Receives pattern change commands
/// * `led_tx` - Sends color commands to RGB LED
///
/// # Behavior
/// Updates LED color based on active pattern at regular intervals
pub async fn run_pattern_executor(
    mut pattern_rx: mpsc::Receiver<(usize, u8, bool)>, // (pattern_index, brightness, paused)
    led_tx: mpsc::Sender<Command>,
) -> Result<()> {
    // Load config to get available patterns
    let config = load_config("config/config.yaml")?;

    if config.lighting_patterns.is_empty() {
        eprintln!("[Pattern Executor] No lighting patterns configured!");
        return Ok(());
    }

    // Initialize with default pattern
    let default_index = config
        .system
        .default_pattern_index
        .min(config.lighting_patterns.len() - 1);
    let initial_pattern = config.lighting_patterns[default_index].clone();
    let mut executor = PatternExecutor::new(
        initial_pattern.clone(),
        led_tx.clone(),
        config.system.default_brightness,
    );

    // Track if pattern updates are paused (for ManualOverride mode)
    let mut updates_paused = false;

    println!(
        "[Pattern Executor] Started with pattern {}/{}: {}",
        default_index + 1,
        config.lighting_patterns.len(),
        initial_pattern.name()
    );

    // Update at 30 FPS for smooth animations
    let mut update_interval = interval(Duration::from_millis(33));

    loop {
        tokio::select! {
            // Handle pattern changes
            Some((pattern_index, brightness, paused)) = pattern_rx.recv() => {
                let index = pattern_index.min(config.lighting_patterns.len() - 1);
                let new_pattern = config.lighting_patterns[index].clone();

                updates_paused = paused;

                if paused {
                    println!("[Pattern Executor] Pattern updates PAUSED (ManualOverride mode)");
                } else {
                    println!(
                        "[Pattern Executor] Switching to pattern {}/{}: {}",
                        index + 1,
                        config.lighting_patterns.len(),
                        new_pattern.name()
                    );
                }

                executor.set_pattern(new_pattern);
                executor.set_brightness(brightness);

                // Only update LED if not paused
                if !paused {
                    executor.update_led().await;
                }
            }

            // Regular color updates for animations
            _ = update_interval.tick() => {
                // Skip updates if paused (ManualOverride mode) or EventDriven pattern
                if !updates_paused && !matches!(executor.current_pattern, LightingPatternConfig::EventDriven) {
                    executor.update_led().await;
                }
            }
        }
    }
}
