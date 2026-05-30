//! LCD Display Task Implementation
//!
//! Spawns an async task to process LCD display commands.

use actuators::LcdController;
use anyhow::Result;
use gpio_core::{Command, load_config};
use tokio::sync::mpsc;

/// Run the LCD display task with command receiver
///
/// # Arguments
/// * `command_rx` - Channel to receive display commands from the controller
///
/// # Behavior
/// Processes LCD commands (text display, clear, backlight control) until channel closes
pub async fn run_lcd_display(mut command_rx: mpsc::Receiver<Command>) -> Result<()> {
    // Load config to get LCD address
    let config = load_config("config/config.yaml")?;
    let lcd_address = config.lcd.i2c_address;

    // Initialize LCD controller
    let mut lcd = LcdController::new(lcd_address, "LCD")?;

    // Clear display and show initial message
    lcd.clear()?;
    lcd.write_line(0, "System Ready")?;
    lcd.write_line(1, "Waiting...")?;

    println!("[LCD] Display task ready!");

    loop {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                // Handle all commands
                match &command {
                    Command::DisplayText { line, text } => {
                        if let Err(e) = lcd.write_line(*line, text) {
                            eprintln!("[LCD] Failed to write text: {}", e);
                        }
                    }
                    Command::ClearDisplay => {
                        if let Err(e) = lcd.clear() {
                            eprintln!("[LCD] Failed to clear display: {}", e);
                        }
                    }
                    Command::DisplayOn => {
                        if let Err(e) = lcd.backlight_on() {
                            eprintln!("[LCD] Failed to turn on display: {}", e);
                        }
                    }
                    Command::DisplayOff => {
                        if let Err(e) = lcd.backlight_off() {
                            eprintln!("[LCD] Failed to turn off display: {}", e);
                        }
                    }
                    _ => {
                        // Ignore non-LCD commands (they're for other actuators)
                    }
                }
            }

            else => {
                break;
            }
        }
    }

    println!("[LCD] Shutting down...");

    // Turn off backlight and clear display on shutdown
    let _ = lcd.backlight_off();
    let _ = lcd.clear();

    Ok(())
}
