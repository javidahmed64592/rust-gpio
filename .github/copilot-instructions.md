# Rust GPIO Event-Driven System - Development Guidelines

## Architecture Principles

This project follows a **strict robotics-inspired event-driven architecture**. All code must respect these foundational rules:

### Event Flow Model

```
[Sensors] → Events → [Controller] → Commands → [Actuators]
```

**NEVER violate this separation:**

- **Sensors emit events only** - No direct control of outputs, no business logic
- **Controller owns all behavior** - Single source of truth for decision making
- **Actuators consume commands only** - No business logic, no reading sensor state

### Communication Pattern

- Use **tokio async** with `mpsc::channel(32)` for events/commands
- Use `broadcast::channel` for shutdown signaling
- Tasks communicate through channels, not shared state
- All tasks must handle shutdown gracefully via `tokio::select!`

## Workspace Structure

```
gpio_core/         - Shared types (Event, Command, SystemState, Config)
sensors/           - Hardware controllers that emit events only
actuators/         - Hardware controllers that consume commands only
controller/        - Business logic and decision making
runner/            - System orchestration and task spawning
config/            - YAML configuration files
rust_gpio_utils/   - Python utility scripts (DHT11 sensor driver)
```

### Crate Responsibilities

**gpio_core** - Protocol layer:
- `Event` enum: All sensor events (MotionDetected, BrightnessButtonPressed, PatternCyclePressed, EnvironmentReading, etc.)
- `Command` enum: All actuator commands (RgbLedOn, SetRgbColor, DisplayText, SetTemperatureLeds, SetHumidityLeds, etc.)
- `RgbColor`: RGB color representation with HSV/lerp utilities
- `LightingPatternConfig`: Pattern configurations (EventDriven, Static, Gradient, Rainbow, Pulse, ColorCycle)
- `SystemState`: Controller's internal state (presence, lighting mode, brightness, pattern index, indicator LED state)
- `SystemConfig`: Configuration loaded from YAML
- `load_config()`: Config loading utility

**sensors** - Hardware input:
- Generic controllers: `ButtonController`, `PirSensorController`, `Dht11Controller`
- Methods return `Option<Event>` - **never** send commands
- Example: `check_motion() -> Option<Event>`, `read_environment() -> Option<Event>`
- DHT11 sensor uses async subprocess to Python utility for reliable readings

**actuators** - Hardware output:
- Generic controllers: `RgbLedController`, `LcdController`, `IndicatorLedController`
- Methods consume commands - **never** emit events
- Example: `set_color(color: RgbColor)`, `set_indicators(low, medium, high, brightness)`
- RGB LED uses hardware PWM (100Hz) for smooth color mixing
- Indicator LEDs use PWM for brightness control (0-100%)

**controller** - Business logic:
- `run_controller(event_rx, led_tx, lcd_tx, pattern_tx, temp_led_tx, humidity_led_tx)` - Main decision loop
- Owns `SystemState` and applies all business rules
- Only place where behavior exists
- Routes commands to appropriate actuator channels

**runner** - Task orchestration:
- Individual task modules: `pir_sensor.rs`, `brightness_button.rs`, `pattern_button.rs`, `lighting_override_button.rs`, `dht11_sensor.rs`, `rgb_led_actuator.rs`, `lcd_display.rs`, `pattern_executor.rs`, `temperature_led_actuator.rs`, `humidity_led_actuator.rs`
- `main.rs`: Bootstrap channels, spawn tasks, handle shutdown
- Minimal business logic

## Configuration Management

All hardware and behavior config lives in `config/config.yaml`.

**Rules:**
- Config is declarative, not procedural
- Each crate loads only the section it needs
- No hardcoded GPIO pins or timing values in code
- Add new config fields to `SystemConfig` struct in `gpio_core`

## Hardware Constraints

**Target:** Raspberry Pi 5 (compatible with Pi 4, 3B+)

**GPIO Libraries:**
- Use `rppal` 0.22.1 for GPIO (digital I/O, PWM, I2C)
- PWM frequency: 100Hz for RGB LED and indicator LED brightness
- Debounce timing: 300ms for buttons
- Polling intervals: 100ms (PIR), 50ms (buttons), 2000ms (DHT11)

**Python Integration:**
- DHT11 sensor uses Python script (`rust_gpio_utils/dht11.py`) via `tokio::process::Command`
- Package manager: `uv` for Python dependencies (gpiozero, lgpio)
- Async subprocess execution prevents blocking event loop

**LCD Timing:**
- Enable pulse: 500μs
- Inter-character delay: 100μs
- 16x2 display with PCF8574 I2C backpack

## Code Style

### Async Patterns

```rust
// Good: Proper shutdown handling
tokio::select! {
    Some(event) = event_rx.recv() => {
        handle_event(event);
    }
    _ = shutdown_rx.recv() => {
        cleanup();
        break;
    }
}

// Bad: No shutdown handling
while let Some(event) = event_rx.recv().await {
    handle_event(event);
}
```

### Event/Command Handling

```rust
// Good: Match exhaustively
match event {
    Event::MotionDetected => { /* logic */ },
    Event::BrightnessButtonPressed => { /* logic */ },
    Event::LightingModeTogglePressed => { /* logic */ },
}

// Bad: Using _ => {} (hides unhandled cases)
```

### Error Handling

```rust
// Good: Explicit error handling
let controller = LedController::new(pin_number, "PIR LED")
    .expect("Failed to initialize LED controller");

// Bad: Silent failures with unwrap() in production paths
```

### Documentation

- All public APIs must have rustdoc comments
- Include `/// # Arguments`, `/// # Behavior`, `/// # Returns` sections
- Generate docs: `cargo doc --workspace --open`

## Common Patterns

### Adding a New Sensor

1. Create hardware controller in `sensors/` (returns `Option<Event>`)
2. Add event variant to `Event` enum in `gpio_core`
3. Create task module in `runner/src/`
4. Update `handle_event()` in `controller/src/controller_impl.rs`
5. Spawn task in `runner/src/main.rs`
6. Add GPIO pin to `config.yaml`

### Adding a New Actuator

1. Create hardware controller in `actuators/` (consumes commands)
2. Add command variant to `Command` enum in `gpio_core`
3. Create task module in `runner/src/`
4. Update controller to emit new command when appropriate
5. Spawn task in `runner/src/main.rs`
6. Add GPIO pin to `config.yaml`

### Controller Logic Changes

**All behavioral changes go in `controller/src/controller_impl.rs`:**

```rust
fn handle_event(
    event: Event,
    state: &mut SystemState,
    config: &SystemConfig,
) -> Vec<Command> {
    // Decision making logic here
    // Return commands to actuators
}
```

**Never put business logic in:**
- Sensor modules (they just read hardware)
- Actuator modules (they just control hardware)
- Runner tasks (they just wire things together)

## Build and Test

```bash
# Build entire workspace
cargo build --workspace

# Run main system (requires hardware)
cargo run --bin runner

# Test individual components
cargo run --bin button-test -- <gpio_pin>
cargo run --bin pir-test -- <gpio_pin>
cargo run --bin lcd-test -- <i2c_address>

# Check for errors
cargo check --workspace

# Generate documentation
cargo doc --workspace --open

# Run with release optimizations
cargo run --release --bin runner
```

## Anti-Patterns to Avoid

❌ **Sensor directly controls actuator**
```rust
// Bad: PIR sensor turning on LED
if motion_detected {
    led_controller.turn_on();
}
```

✅ **Sensor emits event, controller decides**
```rust
// Good: PIR sensor emits event
if motion_detected {
    event_tx.send(Event::MotionDetected).await;
}

// Controller makes decision
if event == Event::MotionDetected && state.lighting_mode == Automatic {
    commands.push(Command::LedOn);
}
```

❌ **Hardcoded GPIO pins**
```rust
const LED_PIN: u8 = 26;
```

✅ **Load from config**
```rust
let pin = config.gpio.led.pir_led_pin;
```

❌ **Business logic in actuators**
```rust
// Bad: LED decides when to blink
impl LedController {
    fn update(&mut self, presence: bool) {
        if presence { self.turn_on(); }
    }
}
```

✅ **Actuators are dumb consumers**
```rust
// Good: LED just executes commands
impl LedController {
    fn turn_on(&mut self) { /* just control hardware */ }
}
```

## Key Design Decisions

### Why Event-Driven?

- **Loose coupling**: Components can be replaced without affecting others
- **Testability**: Can mock sensors/actuators easily
- **Extensibility**: New sensors/actuators plug into existing channels
- **Clarity**: Clear data flow makes debugging easier

### Why Single Controller?

- **Consistency**: All behavior in one place prevents conflicts
- **State management**: Single source of truth for system state
- **Debugging**: Logic errors exist in only one place
- **Simplicity**: No distributed decision making complexity

### Why YAML Config?

- **Hardware flexibility**: Change GPIO pins without recompiling
- **Behavior tuning**: Adjust timeouts/brightness without code changes
- **Documentation**: Config file serves as hardware reference
- **Team sharing**: Version controlled hardware setup

## Current Feature Status

**Completed:**
- PIR motion detection with 120s timeout
- Lighting mode toggle (Automatic / Manual Override)
- Configurable brightness cycling (from YAML array)
- **RGB LED with PWM color control** (3 GPIO pins, 100Hz PWM)
- **6 lighting patterns** (EventDriven, Static, Gradient, Rainbow, Pulse, ColorCycle)
- **Pattern cycle button** to switch between lighting effects
- **DHT11 temperature/humidity sensor** (Python subprocess integration)
- **Temperature indicator LEDs** (low/medium/high with PWM brightness)
- **Humidity indicator LEDs** (low/medium/high with PWM brightness)
- **LCD display** shows pattern name (line 0) and temp/humidity (line 1)
- Pattern executor task with dynamic pattern switching
- Graceful shutdown handling for all tasks
- Comprehensive rustdoc documentation
- String allocation optimizations

**High Priority Next:**
- Smooth color transitions in pattern executor
- Persistent state across restarts (brightness, pattern, mode)
- Configuration hot-reload without restart

See README.md for full feature roadmap.

## Development Workflow

1. **Always** respect the architecture - sensors don't control outputs
2. **Always** update config.yaml for new GPIO pins
3. **Always** add rustdoc comments to public APIs
4. **Always** handle shutdown signals in async tasks
5. **Never** hardcode hardware values
6. **Never** put business logic outside the controller
7. **Never** use global state or shared mutexes

## License

GPL-3.0-or-later - See LICENSE file
