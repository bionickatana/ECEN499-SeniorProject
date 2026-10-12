//! UART communication plugin placeholder.
//!
//! Put UART peripheral ownership, receive parsing, transmit buffering, and the
//! external-device protocol here. Do not put UART command handling in `main.rs`
//! or in the thermocouple plugin.

use crate::{
    config,
    events::AppEvent,
    plugins::{AppContext, Plugin, PollTimer},
};

pub struct UartPlugin {
    initialized: bool,
    poll_timer: PollTimer,
}

impl UartPlugin {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            poll_timer: PollTimer::new(config::UART_POLL_INTERVAL_MS),
        }
    }
}

impl Plugin for UartPlugin {
    fn poll(&mut self, context: &mut AppContext<'_>, now_ms: u64) {
        if !self.poll_timer.should_run(now_ms) {
            return;
        }

        if !self.initialized {
            self.initialized = true;
            return;
        }

        // Future UART work belongs here:
        // - read bytes without blocking
        // - parse protocol frames or ASCII commands
        // - push AppEvent::ReportRequested when the external device asks for data
        // - push AppEvent::UartActivity on RX/TX so the status LED can pulse blue
        let _ = context;
        let _ = AppEvent::ReportRequested;
    }
}
