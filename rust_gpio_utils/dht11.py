"""DHT11 temperature and humidity sensor driver using gpiozero.

This module provides a DHT11 class for reading temperature and humidity
from DHT11 sensors connected to Raspberry Pi GPIO pins.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from collections.abc import Callable

from gpiozero import InputDevice, OutputDevice


class DHT11:
    """DHT11 temperature and humidity sensor driver.

    This implementation uses gpiozero for GPIO control and implements
    the DHT11 one-wire protocol for reading sensor data.

    Attributes:
        MAX_DELAY_COUNT: Maximum delay iterations before timing out.
        BIT_1_DELAY_COUNT: Threshold for distinguishing 1 bits from 0 bits.
        BITS_LEN: Total number of bits in DHT11 data packet (40 bits).
    """

    MAX_DELAY_COUNT = 100
    BIT_1_DELAY_COUNT = 10
    BITS_LEN = 40

    def __init__(self, pin: int, pull_up: bool = False) -> None:  # noqa: FBT001, FBT002
        """Initialize DHT11 sensor.

        Args:
            pin: GPIO pin number where DHT11 data line is connected.
            pull_up: Whether to enable internal pull-up resistor.
        """
        self._pin = pin
        self._pull_up = pull_up

    def read_data(self) -> tuple[float, float]:
        """Read temperature and humidity from DHT11 sensor.

        This method implements the DHT11 one-wire protocol:
        1. Send start signal (pull low for 20ms)
        2. Wait for sensor response
        3. Read 40 bits of data
        4. Verify checksum
        5. Parse temperature and humidity values

        Returns:
            Tuple of (humidity, temperature) as floats.
            Returns (0.0, 0.0) if checksum verification fails or timeout occurs.

        Raises:
            RuntimeError: If GPIO operations fail or sensor times out.
        """
        bit_count = 0
        delay_count = 0
        bits = ""
        gpio = None

        try:
            # -------------- send start --------------
            gpio = OutputDevice(self._pin)
            gpio.off()
            time.sleep(0.02)

            gpio.close()
            gpio = InputDevice(self._pin, pull_up=self._pull_up)

            # -------------- wait response --------------
            # Wait for sensor to pull line low (should happen quickly)
            while gpio.value == 1:
                pass

            # -------------- read data --------------
            while bit_count < self.BITS_LEN:
                # Wait for bit start (low pulse)
                while gpio.value == 0:
                    pass

                # Measure high pulse duration to determine bit value
                delay_count = 0
                while gpio.value == 1:
                    delay_count += 1
                    if delay_count > self.MAX_DELAY_COUNT:
                        break

                bits += "1" if delay_count > self.BIT_1_DELAY_COUNT else "0"
                bit_count += 1

            # -------------- verify --------------
            if len(bits) != self.BITS_LEN:
                error_msg = f"Incomplete data: got {len(bits)} bits, expected {self.BITS_LEN}"
                raise RuntimeError(error_msg)

            humidity_integer = int(bits[0:8], 2)
            humidity_decimal = int(bits[8:16], 2)
            temperature_integer = int(bits[16:24], 2)
            temperature_decimal = int(bits[24:32], 2)
            check_sum = int(bits[32:40], 2)

            _sum = humidity_integer + humidity_decimal + temperature_integer + temperature_decimal

            if check_sum != _sum:
                error_msg = f"Checksum mismatch: calculated {_sum}, received {check_sum}"
                raise RuntimeError(error_msg)

            humidity = float(f"{humidity_integer}.{humidity_decimal}")
            temperature = float(f"{temperature_integer}.{temperature_decimal}")

            # -------------- return --------------
            return humidity, temperature

        finally:
            # Always close GPIO to prevent resource leaks
            if gpio is not None:
                gpio.close()


def read_with_retry(
    dht11: DHT11,
    max_retries: int = 5,
    retry_delay: float = 2.0,
    error_callback: Callable[[int, Exception], None] | None = None,
) -> tuple[float, float]:
    """Read DHT11 sensor with automatic retries.

    Args:
        dht11: DHT11 sensor instance.
        max_retries: Maximum number of read attempts.
        retry_delay: Delay in seconds between retry attempts.
        error_callback: Optional callback function called on each error.
            Receives (attempt_number, exception) as arguments.

    Returns:
        Tuple of (humidity, temperature) as floats.
        Returns (0.0, 0.0) if all attempts fail.
    """
    for attempt in range(1, max_retries + 1):
        try:
            humidity, temperature = dht11.read_data()
            if humidity > 0.0 or temperature > 0.0:
                return humidity, temperature
        except KeyboardInterrupt:
            # Handle graceful shutdown (SIGINT from parent process)
            raise
        except Exception as e:
            if error_callback:
                error_callback(attempt, e)

        # Wait before retry (but not after last attempt)
        # Use shorter sleep increments to be more responsive to interrupts
        if attempt < max_retries:
            # Sleep in smaller chunks to allow faster interrupt response
            remaining = retry_delay
            while remaining > 0:
                sleep_time = min(0.5, remaining)  # Sleep max 0.5s at a time
                time.sleep(sleep_time)
                remaining -= sleep_time

    return 0.0, 0.0


def main() -> None:
    """CLI entry point for DHT11 sensor reading.

    Reads temperature and humidity from DHT11 sensor and outputs JSON
    to stdout for consumption by Rust code.

    Command-line arguments:
        pin: GPIO pin number (required)
        --retries: Number of retry attempts (default: 5)
        --pull-up: Enable internal pull-up resistor (flag)

    Exit codes:
        0: Success - valid reading obtained
        1: Failure - no valid reading after all retries
        2: Invalid arguments
        130: Interrupted by signal (SIGINT)
    """
    parser = argparse.ArgumentParser(
        description="Read temperature and humidity from DHT11 sensor",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s 26                    # Read from GPIO pin 26
  %(prog)s 26 --retries 10       # Retry up to 10 times
  %(prog)s 26 --pull-up          # Enable internal pull-up resistor

Output format (JSON):
  {"temperature": 22.0, "humidity": 45.0}
        """,
    )
    parser.add_argument(
        "pin",
        type=int,
        help="GPIO pin number where DHT11 data line is connected",
    )
    parser.add_argument(
        "--retries",
        type=int,
        default=5,
        help="Number of retry attempts (default: 5)",
    )
    parser.add_argument(
        "--pull-up",
        action="store_true",
        help="Enable internal pull-up resistor",
    )

    try:
        args = parser.parse_args()
    except SystemExit:
        sys.exit(2)

    dht11 = DHT11(args.pin, pull_up=args.pull_up)

    def error_callback(attempt: int, exception: Exception) -> None:
        """Print error to stderr for debugging."""
        print(f"Attempt {attempt} failed: {exception}", file=sys.stderr)

    try:
        humidity, temperature = read_with_retry(
            dht11,
            max_retries=args.retries,
            error_callback=error_callback,
        )

        if humidity > 0.0 or temperature > 0.0:
            result = {
                "temperature": temperature,
                "humidity": humidity,
            }
            print(json.dumps(result))
            sys.exit(0)
        else:
            print("Error: Failed to read valid data from sensor", file=sys.stderr)
            sys.exit(1)
    except KeyboardInterrupt:
        # Graceful shutdown on SIGINT (Ctrl+C or parent process shutdown)
        sys.exit(130)  # 128 + SIGINT(2) = standard exit code for SIGINT


if __name__ == "__main__":
    main()
