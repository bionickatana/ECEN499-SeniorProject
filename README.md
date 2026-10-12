# sw_demo

Firmware for the RP2350-based demo board. The current implemented hardware paths
cover the onboard WS2812/NeoPixel status LED, USB CDC debug output, and MAX6675
SPI thermocouple reads for channel 0. UART communication and report formatting
are still isolated as placeholders so separate contributors can implement those
pieces without editing a giant `main.rs`.

## Project Goals

- Keep `main.rs` small and boring.
- Isolate UART, SPI thermocouple reads, reporting, and status LED behavior.
- Use a simple cooperative plugin loop instead of an RTOS.
- Keep the current `rp235x-hal` setup instead of migrating HAL ecosystems.
- Make hardware configuration easy to find and update.

## Build And Flash

Install the ARM target:

```sh
rustup target add thumbv8m.main-none-eabihf
```

Build:

```sh
cargo build
```

Flash and run with the configured `picotool` runner:

```sh
cargo run
```

The default target is configured in `.cargo/config.toml`.

## Architecture

This project uses statically linked "plugins". They are normal Rust modules, not
runtime-loaded plugins. The word plugin means a subsystem with a narrow owner,
small public interface, and a non-blocking `poll()` method.

Current layout:

```text
src/
  main.rs                 firmware entry point only
  app.rs                  cooperative plugin composition and event handling
  board.rs                RP2350/Waveshare hardware setup
  config.rs               board constants and placeholder pin assignments
  events.rs               small fixed event queue shared by plugins

  plugins/
    mod.rs                Plugin trait, AppContext, and PollTimer
    status_led.rs         NeoPixel status LED renderer
    uart.rs               UART placeholder
    thermocouples.rs      SPI thermocouple reader
    debug_console.rs      USB CDC debug output
    reporter.rs           report-generation placeholder

  services/
    mod.rs
    status.rs             shared system status and LED pattern mapping
    sensor_data.rs        shared thermocouple readings
```

## File Ownership

`main.rs` should only initialize the board and start the app. Do not add UART,
SPI, reporting, or LED logic here.

`board.rs` owns hardware setup. Add UART construction here when that protocol is
implemented, and update the SPI setup here when the thermocouple pinout changes
or additional chip-selects are wired. If a pin assignment changes, update
`config.rs` and the matching setup code in `board.rs`.

`app.rs` owns plugin composition and event handling. It creates plugins, builds
`AppContext`, polls every plugin continuously, and applies high-level
event-to-status decisions. Individual plugins own their poll cadence with their
own `PollTimer` fields.

`plugins/uart.rs` owns UART protocol work. Put RX parsing, TX buffering, and
external-device command handling there.

`plugins/thermocouples.rs` owns SPI thermocouple polling. Put chip-select
sequencing, SPI transactions, thermocouple IC decoding, and fault detection
there.

`plugins/reporter.rs` owns report formatting. It should read shared sensor data
and prepare output when UART requests data. It should not directly read SPI.

`plugins/status_led.rs` owns WS2812 writes and LED animation rendering. Other
plugins should change `StatusService`; they should not write NeoPixel colors
directly.

`services/status.rs` owns the mapping from application status to LED pattern.
Add new status states or custom LED patterns there.

`services/sensor_data.rs` owns the latest thermocouple readings. The
thermocouple plugin writes this data, and the reporter reads it.

## Cooperative Plugin Loop

The firmware is designed like a tiny cooperative scheduler. The main loop runs
continuously without a fixed sleep. Each plugin receives the current monotonic
millisecond timestamp and returns immediately when called before its configured
interval has elapsed.

```text
loop:
  read current time from hardware timer
  poll status service timeout state
  poll UART plugin
  poll USB debug console device
  poll thermocouple plugin
  poll debug console output
  poll reporter plugin
  apply events to shared status
  poll status LED plugin
  clear events
```

Plugin poll intervals are configured in `src/config.rs` and enforced inside each
plugin with `PollTimer`. An interval of `0` means the plugin runs every loop.
The current defaults are:

```text
STATUS_LED_POLL_INTERVAL_MS          5 ms
THERMOCOUPLE_POLL_INTERVAL_MS        1000 ms
DEBUG_CONSOLE_REPORT_INTERVAL_MS     1000 ms
UART_POLL_INTERVAL_MS                0 ms / every loop
REPORTER_POLL_INTERVAL_MS            0 ms / every loop
UART_ACTIVITY_DURATION_MS            150 ms
```

The MAX6675 is read once per second by default. USB CDC device polling still
runs every loop so the host connection remains responsive; only formatted debug
console output is rate-limited.

Plugin rules:

- Keep `poll()` short.
- Do not wait in long blocking loops.
- Store plugin progress in the plugin struct.
- Store plugin timing state in the plugin struct with `PollTimer`.
- Split slow work into states and advance one state per `poll()`.
- Communicate with other plugins through `EventQueue` and services, not direct
  cross-plugin calls.

This keeps the project simple now while leaving room to migrate to RTIC or
Embassy later if the application needs a real task runtime.

## Status LED

The onboard NeoPixel is currently connected to GPIO16 and driven with PIO0 state
machine 0.

Default status mapping:

```text
Booting       -> yellow breathing
Operating     -> green breathing
Error         -> red flashing
UART activity -> blue pulse
```

To add a new LED behavior, update `LedPattern` or `SystemStatus` in
`src/services/status.rs`. Keep low-level WS2812 timing changes in
`src/plugins/status_led.rs` or `src/board.rs`.

## Hardware Configuration

Known current hardware:

```text
MCU/board:        Waveshare RP2350-Zero style board
Crystal:          12 MHz
Status NeoPixel:  GPIO16
Target:           thumbv8m.main-none-eabihf
```

Placeholder hardware configuration is documented in `src/config.rs`:

```text
UART_TX_PIN
UART_RX_PIN
UART_BAUD
STATUS_LED_POLL_INTERVAL_MS
THERMOCOUPLE_POLL_INTERVAL_MS
DEBUG_CONSOLE_REPORT_INTERVAL_MS
UART_POLL_INTERVAL_MS
REPORTER_POLL_INTERVAL_MS
UART_ACTIVITY_DURATION_MS
SPI_SCK_PIN
SPI_MOSI_PIN
SPI_MISO_PIN
SPI_BAUD_HZ
THERMOCOUPLE_CS_PINS
```

These values are placeholders only. When the schematic/pinout is finalized,
update `config.rs` and implement the matching peripheral setup in `board.rs`.

## Contributor Guide

If you are implementing UART:

- Start in `src/plugins/uart.rs`.
- Add hardware initialization to `src/board.rs`.
- Add final pins and baud rate to `src/config.rs`.
- Emit `AppEvent::ReportRequested` when the external device asks for sensor
  data.
- Emit `AppEvent::UartActivity` on RX or TX activity.

If you are implementing thermocouple reads:

- Start in `src/plugins/thermocouples.rs`.
- Update SPI and CS setup in `src/board.rs`.
- Update the SPI timing and CS pin constants in `src/config.rs`.
- Write readings into `SensorDataService`.
- Emit `AppEvent::SensorUpdated` or `AppEvent::SensorFault`.
- Keep sensor read cadence in the thermocouple plugin's `PollTimer` and
  `THERMOCOUPLE_POLL_INTERVAL_MS`; MAX6675 reads must not run faster than four
  times per second.

If you are implementing reporting:

- Start in `src/plugins/reporter.rs`.
- Read sensor values from `SensorDataService`.
- Keep formatting/protocol code out of the thermocouple plugin.
- Coordinate UART output with the UART owner once the protocol is defined.

If you are changing status LED behavior:

- Start in `src/services/status.rs` for status-to-pattern mapping.
- Use `src/plugins/status_led.rs` only for animation rendering or WS2812 writes.

## Future Work

- Finalize UART protocol.
- Finalize board pinout.
- Implement SPI thermocouple reads for four chip-selects.
- Implement UART TX/RX buffering.
- Decide whether the cooperative loop is sufficient or whether RTIC/Embassy is
  worth adopting later.
