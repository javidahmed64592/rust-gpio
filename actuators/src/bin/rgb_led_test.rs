//! RGB LED Test Binary
//!
//! Test RGB LED by specifying GPIO pin numbers for red, green, and blue channels.
//! This allows testing that RGB LEDs are wired correctly and demonstrates color mixing.

use anyhow::{Context, Result};
use gpio_core::RgbColor;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <red_pin> <green_pin> <blue_pin>", args[0]);
        eprintln!("\nExample:");
        eprintln!(
            "  {} 23 24 25    # Test RGB LED with R=GPIO23, G=GPIO24, B=GPIO25",
            args[0]
        );
        eprintln!("\nFrom config.yaml:");
        eprintln!("  {} 23 24 25    # System RGB LED", args[0]);
        std::process::exit(1);
    }

    let red_pin: u8 = args[1]
        .parse()
        .context("Red pin number must be a valid integer (0-27)")?;
    let green_pin: u8 = args[2]
        .parse()
        .context("Green pin number must be a valid integer (0-27)")?;
    let blue_pin: u8 = args[3]
        .parse()
        .context("Blue pin number must be a valid integer (0-27)")?;

    println!("=== RGB LED Test ===");
    println!("Testing RGB LED:");
    println!("  Red channel:   GPIO {}", red_pin);
    println!("  Green channel: GPIO {}", green_pin);
    println!("  Blue channel:  GPIO {}", blue_pin);
    println!("\nThis test will demonstrate:");
    println!("  1. Individual color channels (Red, Green, Blue)");
    println!("  2. Predefined colors (Green, Blue, Red, Orange)");
    println!("  3. Brightness levels (25%, 50%, 75%, 100%)");
    println!("  4. Error blink pattern");
    println!("\nPress Ctrl+C to exit early\n");

    // Initialize RGB LED
    let mut led = actuators::RgbLedController::new(red_pin, green_pin, blue_pin, "RGB LED Test")?;

    // Ensure LED starts off
    led.turn_off();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Test 1: Individual color channels
    println!("\n=== Test 1: Individual Color Channels ===");

    println!("\n[1/3] Red channel only (100%)");
    led.set_color(RgbColor::new(100, 0, 0));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[2/3] Green channel only (100%)");
    led.set_color(RgbColor::new(0, 100, 0));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[3/3] Blue channel only (100%)");
    led.set_color(RgbColor::new(0, 0, 100));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    led.turn_off();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 2: Predefined colors
    println!("\n=== Test 2: Predefined Colors ===");

    println!("\n[1/4] Green (normal operation)");
    led.set_color(RgbColor::green());
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[2/4] Blue (motion detected)");
    led.set_color(RgbColor::blue());
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[3/4] Orange (system busy)");
    led.set_color(RgbColor::orange());
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[4/4] Red (error)");
    led.set_color(RgbColor::red());
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    led.turn_off();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 3: Brightness levels
    println!("\n=== Test 3: Brightness Levels (Green Color) ===");
    led.set_color(RgbColor::green());

    for brightness in [25, 50, 75, 100] {
        println!("\nBrightness: {}%", brightness);
        led.set_brightness(brightness);
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    }

    led.turn_off();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 4: Mixed colors
    println!("\n=== Test 4: Mixed Colors ===");

    println!("\n[1/5] Yellow (Red + Green)");
    led.set_color(RgbColor::new(100, 100, 0));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[2/5] Cyan (Green + Blue)");
    led.set_color(RgbColor::new(0, 100, 100));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[3/5] Magenta (Red + Blue)");
    led.set_color(RgbColor::new(100, 0, 100));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[4/5] White (All channels)");
    led.set_color(RgbColor::new(100, 100, 100));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("[5/5] Dim white (50% all channels)");
    led.set_color(RgbColor::new(50, 50, 50));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    led.turn_off();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 5: Error blink pattern
    println!("\n=== Test 5: Error Blink Pattern ===");
    println!("\nBlinking red 3 times...");
    led.set_color(RgbColor::green());
    led.set_brightness(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    led.blink_error(3);
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Final cleanup
    led.turn_off();

    println!("\n=== Test Complete ===");
    println!("✓ All color tests passed!");
    println!("\nIf you saw all the colors correctly, your RGB LED is wired properly:");
    println!("  • Red channel on GPIO {}", red_pin);
    println!("  • Green channel on GPIO {}", green_pin);
    println!("  • Blue channel on GPIO {}", blue_pin);
    println!("\nNote: If colors appear wrong (e.g., red shows as blue),");
    println!("      check your wiring and verify pin assignments in config.yaml\n");

    Ok(())
}
