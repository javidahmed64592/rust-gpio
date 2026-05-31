//! DHT11 Temperature and Humidity Sensor Controller
//!
//! Provides a hardware abstraction for DHT11 environmental sensors.
//! Reads temperature (Celsius) and relative humidity (percentage).
//!
//! Uses a Python utility script (gpiozero-based) via subprocess for reliable hardware communication.
//! This approach leverages the proven gpiozero library while maintaining the Rust event-driven architecture.

use anyhow::Result;
use gpio_core::Event;
use tokio::process::Command;

/// DHT11 temperature and humidity sensor controller
///
/// Uses Python subprocess to call rust-gpio-dht11 CLI utility for reliable readings.
pub struct Dht11Controller {
    pin_number: u8,
    label: String,
    last_temperature: Option<f32>,
    last_humidity: Option<f32>,
}

impl Dht11Controller {
    /// Initialize DHT11 sensor controller with specified GPIO pin
    ///
    /// # Arguments
    /// * `pin_number` - GPIO pin number the DHT11 data line is connected to
    /// * `label` - Label for logging (e.g., "DHT11", "Environment Sensor")
    pub fn new(pin_number: u8, label: &str) -> Result<Self> {
        println!("[{}] Initialized on GPIO pin: {}", label, pin_number);
        println!("[{}] Sensor type: DHT11 (temperature + humidity)", label);
        println!(
            "[{}] Using gpiozero-based Python utility via uv subprocess",
            label
        );
        println!(
            "[{}] Note: Requires uv virtual environment (run 'uv sync' in project root)",
            label
        );

        Ok(Self {
            pin_number,
            label: label.to_string(),
            last_temperature: None,
            last_humidity: None,
        })
    }

    /// Read temperature and humidity from the DHT11 sensor
    ///
    /// # Returns
    /// - `Some(Event::EnvironmentReading)` if reading is successful
    /// - `None` if reading fails (sensor not ready, checksum error, timing issues, etc.)
    ///
    /// # Behavior
    /// Calls rust-gpio-dht11 Python utility via subprocess for reliable hardware access.
    /// DHT11 sensor requires at least 1 second between readings for stability.
    /// Failed readings are logged but don't panic - the sensor can be temperamental.
    pub async fn read_environment(&mut self) -> Option<Event> {
        // Execute Python utility using uv run (async)
        let output = match Command::new("uv")
            .arg("run")
            .arg("rust-gpio-dht11")
            .arg(self.pin_number.to_string())
            .arg("--retries")
            .arg("5")
            .output()
            .await
        {
            Ok(output) => output,
            Err(e) => {
                eprintln!(
                    "[{}] Failed to execute uv command: {} (ensure 'uv sync' was run)",
                    self.label, e
                );
                return None;
            }
        };

        // Check if command succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[{}] Python script failed: {}", self.label, stderr.trim());
            return None;
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let reading: Result<serde_json::Value, _> = serde_json::from_str(stdout.trim());

        match reading {
            Ok(json) => {
                let temp = json["temperature"].as_f64()? as f32;
                let humidity = json["humidity"].as_f64()? as f32;

                self.last_temperature = Some(temp);
                self.last_humidity = Some(humidity);

                println!(
                    "[{}] Temperature: {:.1}\u{00b0}C, Humidity: {:.1}%",
                    self.label, temp, humidity
                );

                Some(Event::EnvironmentReading {
                    temperature_celsius: temp,
                    humidity_percent: humidity,
                })
            }
            Err(e) => {
                eprintln!(
                    "[{}] Failed to parse JSON output: {} (output: {})",
                    self.label,
                    e,
                    stdout.trim()
                );
                None
            }
        }
    }

    /// Get the last successful reading without polling the sensor
    ///
    /// # Returns
    /// The most recent reading as (temperature, humidity), or None if no reads yet
    pub fn last_reading(&self) -> Option<(f32, f32)> {
        match (self.last_temperature, self.last_humidity) {
            (Some(temp), Some(humidity)) => Some((temp, humidity)),
            _ => None,
        }
    }

    /// Get the GPIO pin number this controller is using
    pub fn pin_number(&self) -> u8 {
        self.pin_number
    }
}
