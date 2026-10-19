//! RS-485 node plugin (not yet wired into `board.rs` / `app.rs` / `mod.rs`).
//!
//! This implements the ladder-side polling protocol: the PLC master sends a
//! 4-byte request (`D100` node address, `D101` instruction), and a node that
//! matches `NODE_ID` replies with a 12-byte frame (`[RS D100 K2 D20 K6]`):
//! node address, four data words, and a wrapping checksum of those words.
//!
//! `init_rs485` owns UART1 + GPIO14 setup and should move into `board.rs`'s
//! `Board::init` when this is integrated, returning an `Rs485Hardware` field
//! on `Board` the same way `ThermocoupleHardware` does today. `Rs485Node`
//! should move into `AppContext`/`App` as a plugin implementing the `Plugin`
//! trait at that point; `poll` here deliberately takes no `AppContext` yet
//! because the response payload (`p1`..`p4`) still needs to be wired to a
//! real data source instead of the placeholder constants below.

use embedded_hal::digital::OutputPin;
use embedded_io::{Read, Write};
use rp235x_hal::{
    fugit::HertzU32,
    gpio::{
        FunctionNull, FunctionSioOutput, FunctionUart, Pin, PullDown, PullNone,
        bank0::{Gpio4, Gpio5, Gpio14},
    },
    pac,
    uart::{DataBits, Enabled, StopBits, UartConfig, UartPeripheral},
};

/// Match D10 = K2 in the PLC ladder (Node 0, 1, 2).
pub const NODE_ID: u16 = 2;

const UART_BAUD_HZ: u32 = 9600;

/// Cycles to wait before/after driving the RS-485 bus, so the last stop bit
/// clears and the transceiver has settled before a byte is pushed onto the
/// wire. Tuned empirically against the ladder's polling cadence, not derived
/// from the UART bit rate.
const TURNAROUND_DELAY_CYCLES: u32 = 50_000;
const DRIVE_SETTLE_DELAY_CYCLES: u32 = 20_000;
const TAIL_GUARD_DELAY_CYCLES: u32 = 15_000;

pub type Rs485Uart = UartPeripheral<
    Enabled,
    pac::UART1,
    (
        Pin<Gpio4, FunctionUart, PullNone>,
        Pin<Gpio5, FunctionUart, PullNone>,
    ),
>;

pub type Rs485DirectionPin = Pin<Gpio14, FunctionSioOutput, PullDown>;

pub struct Rs485Hardware {
    pub uart: Rs485Uart,
    pub rts: Rs485DirectionPin,
}

/// Sets up UART1 on GP4/GP5 and GP14 as the RS-485 DE/!RE direction pin.
///
/// Mirrors `board.rs`'s `init_*` helpers: takes raw peripherals/pins, returns
/// the hardware resource struct the matching plugin will own.
pub fn init_rs485(
    uart1: pac::UART1,
    resets: &mut pac::RESETS,
    peripheral_clock_freq: HertzU32,
    tx_pin: Pin<Gpio4, FunctionNull, PullDown>,
    rx_pin: Pin<Gpio5, FunctionNull, PullDown>,
    direction_pin: Pin<Gpio14, FunctionNull, PullDown>,
) -> Rs485Hardware {
    let uart_pins = (
        tx_pin.reconfigure::<FunctionUart, PullNone>(),
        rx_pin.reconfigure::<FunctionUart, PullNone>(),
    );

    let mut rts = direction_pin.into_push_pull_output();
    let _ = rts.set_low(); // Start in listening/receive mode.

    let uart = UartPeripheral::new(uart1, uart_pins, resets)
        .enable(
            UartConfig::new(
                HertzU32::from_raw(UART_BAUD_HZ),
                DataBits::Eight,
                None,
                StopBits::One,
            ),
            peripheral_clock_freq,
        )
        .unwrap();

    Rs485Hardware { uart, rts }
}

/// Owns the RS-485 request/response state machine for one node.
pub struct Rs485Node {
    hardware: Rs485Hardware,
    rx_buf: [u8; 4],
    rx_len: usize,
}

impl Rs485Node {
    pub fn new(hardware: Rs485Hardware) -> Self {
        Self {
            hardware,
            rx_buf: [0; 4],
            rx_len: 0,
        }
    }

    /// Runs one non-blocking step: consume at most one incoming byte, and
    /// reply once a full 4-byte request addressed to `NODE_ID` has arrived.
    pub fn poll(&mut self) {
        let mut byte = [0u8; 1];
        if self.hardware.uart.read(&mut byte).is_err() {
            return;
        }

        self.rx_buf[self.rx_len] = byte[0];
        self.rx_len += 1;

        if self.rx_len < 4 {
            return;
        }

        let requested_address = u16::from_le_bytes([self.rx_buf[0], self.rx_buf[1]]);
        let _command = u16::from_le_bytes([self.rx_buf[2], self.rx_buf[3]]);
        self.rx_len = 0;

        if requested_address != NODE_ID {
            return;
        }

        self.respond();
    }

    fn respond(&mut self) {
        cortex_m::asm::delay(TURNAROUND_DELAY_CYCLES);
        let _ = self.hardware.rts.set_high();
        cortex_m::asm::delay(DRIVE_SETTLE_DELAY_CYCLES);

        // Placeholder payload. Replace with real D21-D24 data once this
        // plugin is wired into `AppContext`/`SensorDataService`.
        let p1: u16 = 10;
        let p2: u16 = 20;
        let p3: u16 = 30;
        let p4: u16 = 40;
        let checksum = p1.wrapping_add(p2).wrapping_add(p3).wrapping_add(p4);

        // Exactly 12 bytes: 6 words matching [RS D100 K2 D20 K6].
        // D20: Node Address
        // D21-D24: Data words
        // D25: Checksum
        let mut response = [0u8; 12];
        response[0..2].copy_from_slice(&NODE_ID.to_le_bytes());
        response[2..4].copy_from_slice(&p1.to_le_bytes());
        response[4..6].copy_from_slice(&p2.to_le_bytes());
        response[6..8].copy_from_slice(&p3.to_le_bytes());
        response[8..10].copy_from_slice(&p4.to_le_bytes());
        response[10..12].copy_from_slice(&checksum.to_le_bytes());

        let _ = self.hardware.uart.write_all(&response);
        let _ = self.hardware.uart.flush();

        cortex_m::asm::delay(TAIL_GUARD_DELAY_CYCLES);
        let _ = self.hardware.rts.set_low();
    }
}
