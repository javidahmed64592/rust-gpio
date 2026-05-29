//! LED implementation module

use anyhow::Result;
use gpio_core::{Command, load_config};
use rppal::gpio::{Gpio, OutputPin};
use tokio::sync::mpsc;

struct LedController {
    pin: OutputPin,
}

impl LedController {
    /// Initialize LED controller with specified GPIO pin
    fn new(pin_number: u8) -> Result<Self> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_number)?.into_output();
        println!("LED initialized on GPIO pin: {}", pin_number);
        Ok(Self { pin })
    }

    /// Turn LED on
    fn turn_on(&mut self) {
        self.pin.set_high();
        println!("LED: ON");
    }

    /// Turn LED off
    fn turn_off(&mut self) {
        self.pin.set_low();
        println!("LED: OFF");
    }

    /// Set brightness using PWM (0-100)
    /// TODO: Implement proper software PWM for brightness control
    fn set_brightness(&mut self, level: u8) {
        if level > 0 {
            self.turn_on();
        } else {
            self.turn_off();
        }
        println!("LED brightness: {}%", level);
    }
}

/// Run the LED actuator with a command receiver channel
pub async fn run_led_actuator(mut command_rx: mpsc::Receiver<Command>) -> Result<()> {
    // Load config to get LED pin
    let config = load_config("config/config.yaml")?;
    let led_pin = config.gpio.led.pir_led_pin;

    // Initialize LED controller
    let mut led = LedController::new(led_pin)?;

    // Ensure LED starts off
    led.turn_off();
    println!("LED ready to receive commands!");

    // Process commands from the channel
    while let Some(command) = command_rx.recv().await {
        match command {
            Command::LedOn => led.turn_on(),
            Command::LedOff => led.turn_off(),
            Command::SetBrightness(level) => led.set_brightness(level),
            _ => {}
        }
    }

    // Clean shutdown - turn off LED
    led.turn_off();
    println!("LED Actuator shutting down...");

    Ok(())
}
