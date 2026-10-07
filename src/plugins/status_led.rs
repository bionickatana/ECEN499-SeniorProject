//! Status LED plugin.
//!
//! This module owns LED animation policy and WS2812 writes. Other plugins should
//! change `StatusService`; they should not know how NeoPixel timing works.

use rp235x_hal as hal;

use hal::{
    pac,
    pio::{Running, SM0, StateMachine, Tx},
};

use crate::{
    plugins::{AppContext, Plugin},
    services::status::{Color, LedPattern},
};

pub type Ws2812Tx = Tx<(pac::PIO0, SM0)>;
pub type Ws2812StateMachine = StateMachine<(pac::PIO0, SM0), Running>;

pub struct Ws2812Hardware {
    tx: Ws2812Tx,
    _sm: Ws2812StateMachine,
}

impl Ws2812Hardware {
    pub fn new(tx: Ws2812Tx, sm: Ws2812StateMachine) -> Self {
        Self { tx, _sm: sm }
    }

    pub fn write(&mut self, color: Color) {
        // WS2812 expects GRB. Because the PIO state machine shifts left, the 24
        // useful bits are placed in bits 31..8 of the FIFO word.
        let encoded =
            ((color.green as u32) << 24) | ((color.red as u32) << 16) | ((color.blue as u32) << 8);

        let _ = self.tx.write(encoded);
    }
}

pub struct StatusLedPlugin {
    hardware: Ws2812Hardware,
    phase: u8,
    breathing_phase: u8,
}

impl StatusLedPlugin {
    pub fn new(hardware: Ws2812Hardware) -> Self {
        Self {
            hardware,
            phase: 0,
            breathing_phase: 0,
        }
    }
}

impl Plugin for StatusLedPlugin {
    fn poll(&mut self, context: &mut AppContext<'_>) {
        let pattern = context.status.current().led_pattern();
        let color = render_pattern(pattern, self.phase, self.breathing_phase);

        self.hardware.write(color);
        self.phase = self.phase.wrapping_add(1);
        if self.phase % 2 == 0 {
            self.breathing_phase = self.breathing_phase.wrapping_add(1);
        }
    }
}

fn render_pattern(pattern: LedPattern, phase: u8, breathing_phase: u8) -> Color {
    match pattern {
        LedPattern::Off => Color::BLACK,
        LedPattern::Solid(color) => color.scaled(32),
        LedPattern::Breathing(color) => color.scaled(breathing_brightness(breathing_phase)),
        LedPattern::Flashing(color) => {
            if phase < 128 {
                color.scaled(48)
            } else {
                Color::BLACK
            }
        }
        LedPattern::Pulse(color) => color.scaled(pulse_brightness(phase)),
    }
}

fn breathing_brightness(phase: u8) -> u8 {
    let triangle = if phase < 128 { phase } else { 255 - phase };
    33 - (triangle / 4)
}

fn pulse_brightness(phase: u8) -> u8 {
    if phase < 64 { 64 - phase } else { 0 }
}
