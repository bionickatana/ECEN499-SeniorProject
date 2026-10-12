//! Central firmware configuration.
//!
//! Contributors should put board-level constants here instead of scattering pin
//! numbers, rates, or timing values through plugin code. Hardware setup code in
//! `board.rs` should consume these constants.

/// Waveshare RP2350-Zero crystal frequency.
pub const XTAL_FREQ_HZ: u32 = 12_000_000;

/// Onboard WS2812 NeoPixel data pin.
pub const NEOPIXEL_PIN: u8 = 16;

/// WS2812 PIO timing.
///
/// The PIO program uses 10 PIO cycles per WS2812 bit. An 8 MHz PIO clock gives
/// 1.25 us per bit, which is the standard 800 kHz WS2812 data rate.
pub const WS2812_PIO_FREQ_HZ: u32 = 8_000_000;

/// Status LED animation cadence.
pub const STATUS_LED_POLL_INTERVAL_MS: u32 = 5;

/// MAX6675 thermocouple read interval.
pub const THERMOCOUPLE_POLL_INTERVAL_MS: u32 = 1_000;

/// USB debug console reporting interval.
pub const DEBUG_CONSOLE_REPORT_INTERVAL_MS: u32 = 1_000;

/// UART and reporter run every loop so they can react to events immediately.
pub const UART_POLL_INTERVAL_MS: u32 = 0;
pub const REPORTER_POLL_INTERVAL_MS: u32 = 0;

/// UART activity status pulse duration.
pub const UART_ACTIVITY_DURATION_MS: u32 = 150;

/// Default UART baud rate placeholder.
///
/// The UART contributor should update this once the external-device protocol is
/// defined.
pub const UART_BAUD: u32 = 115_200;

/// Placeholder UART pin assignment.
///
/// These are documentation placeholders only. `board.rs` should be updated when
/// the final schematic assigns UART pins.
pub const UART_TX_PIN: u8 = 0;
pub const UART_RX_PIN: u8 = 1;

/// Placeholder SPI pin assignment for the thermocouple bus.
///
/// The MAX6675 boards expose SCK, CS, and SOMI. GP3 is configured as SPI TX so
/// the RP2350 SPI peripheral can clock reads, but it is not connected to the
/// MAX6675 boards.
pub const SPI_SCK_PIN: u8 = 2;
pub const SPI_MOSI_PIN: u8 = 3;
pub const SPI_MISO_PIN: u8 = 0;

/// MAX6675 SPI clock rate.
pub const SPI_BAUD_HZ: u32 = 0_100_000;

/// Chip-select pins for thermocouple devices. Channels 0 and 1 are currently
/// populated by the MAX6675 implementation.
pub const THERMOCOUPLE_CS_PINS: [u8; 4] = [1, 5, 6, 7];
