//! UART communication plugin: RS-485 ladder-polling protocol.
//!
//! Put UART peripheral ownership, receive parsing, transmit buffering, and the
//! external-device protocol here. Do not put UART command handling in `main.rs`
//! or in the thermocouple plugin.
//!
//! `UartPlugin` implements the RS-485 node side of the protocol: the PLC
//! master sends a 4-byte request (`D100` node address, `D101` instruction),
//! and a node that matches `NODE_ID` replies with a 12-byte frame
//! (`[RS D100 K2 D20 K6]`): node address, four data words, and a wrapping
//! checksum.
//!
//! `UartPlugin` is not yet handed real hardware: `self.hardware` starts as
//! `None` and `poll` is a no-op until something calls `set_hardware`.
//! `init_rs485` sets up UART1 on GP4/GP5 and GP14 (RS-485 DE/!RE) and should
//! move into `board.rs`'s `Board::init` the same way `init_thermocouple_spi`
//! does; whoever wires that in then calls `uart.set_hardware(...)` from
//! `app.rs` after `Board::init`. The `p1..p4` response payload is still a
//! placeholder pending a real data source (`SensorDataService`).

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

use crate::{
    config,
    events::AppEvent,
    plugins::{AppContext, Plugin, PollTimer},
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
#[allow(dead_code)]
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

/// Owns the RS-485 request/response state machine for this node.
pub struct UartPlugin {
    poll_timer: PollTimer,
    hardware: Option<Rs485Hardware>,
    rx_buf: [u8; 4],
    rx_len: usize,
}

impl UartPlugin {
    pub const fn new() -> Self {
        Self {
            poll_timer: PollTimer::new(config::UART_POLL_INTERVAL_MS),
            hardware: None,
            rx_buf: [0; 4],
            rx_len: 0,
        }
    }

    /// Hands the plugin its UART1/GPIO14 hardware once `board.rs` owns it.
    /// Until this is called, `poll` is a no-op.
    #[allow(dead_code)]
    pub fn set_hardware(&mut self, hardware: Rs485Hardware) {
        self.hardware = Some(hardware);
    }
}

impl Plugin for UartPlugin {
    fn poll(&mut self, context: &mut AppContext<'_>, now_ms: u64) {
        if !self.poll_timer.should_run(now_ms) {
            return;
        }

        let hardware = match &mut self.hardware {
            Some(hardware) => hardware,
            None => return,
        };

        let mut byte = [0u8; 1];
        if hardware.uart.read(&mut byte).is_err() {
            return;
        }

        context.events.push(AppEvent::UartActivity);

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

        respond(hardware);
        context.events.push(AppEvent::ReportRequested);
    }
}

fn respond(hardware: &mut Rs485Hardware) {
    cortex_m::asm::delay(TURNAROUND_DELAY_CYCLES);
    let _ = hardware.rts.set_high();
    cortex_m::asm::delay(DRIVE_SETTLE_DELAY_CYCLES);

    // Placeholder payload. Replace with real D21-D24 data once this plugin
    // reads from `SensorDataService` via `AppContext`.
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

    let _ = hardware.uart.write_all(&response);
    let _ = hardware.uart.flush();

    cortex_m::asm::delay(TAIL_GUARD_DELAY_CYCLES);
    let _ = hardware.rts.set_low();
}
