Temperature Monitor
===================
John D. Corbett
June, 2024

Illustrates how to program a Teensy 4.0 in Rust to
   - Read a [MCP9808 temperature sensor](https://www.microchip.com/en-us/product/mcp9808)
   - Control an LED
   - Read momentary push buttons
   - Report information over USB-Serial

The LED on the Teensy 4.0 lights when the temperature reported by the MCP9808
exceeds a threshold, initially 30 ℃. Note, this temperature is a bit warmer
than a typical office environment, but a bit cooler than body temperature,
so it is possible to test the system by merely pinching the sensor with a bare
hand.

The threshold can be increased or decreased by a degree Celsius with two 
momentary push buttons.

Error reporting can be demonstrated by disconnecting one or more of the sensor wires.

Setup
-----

MCP9808 - Temperature Sensor
   - A0 - Teensy 4.0: G
   - A1 - Teensy 4.0: G
   - A2 - Teensy 4.0: G
   - SCL - Teensy 4.0: 16
   - SDA - Teensy 4.0: 17
   - VCC - Teensy 4.0: 3.3V
   - GND - Teensy 4.0: G

Up Button - Momentary Switch
   - 1 - Teensy 4.0: 10
   - 2 - Teensy 4.0: 12

Down Button - Momentary Switch
   - 1 - Teensy 4.0: 11
   - 2 - Teensy 4.0: 12

The Teensy 4.0 will appear at /dev/ttyACM<n> on Unix-like platforms, e.g. /dev/ttyACM0;
or /dev/cu.usbmodem<n> on Mac.

The Teensy 4.0 should report the temperature and threshold periodically.


To build
--------
   cargo build --release

To convert the executable from elf to Intel hex format
   cargo objcopy --release -- -O ihex t4.hex

To download the firmware into the Teensy 4.0
   teensy_loader_cli --mcu=TEENSY40 -v -w t4.hex

Deviation from Requirements
---------------------------

- I wasn't able to get USB-Serial input working, so I added the push buttons instead.

- I didn't have time to create a mock interface for working without actual hardware.
