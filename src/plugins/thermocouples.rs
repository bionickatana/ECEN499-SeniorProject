//! SPI thermocouple plugin placeholder.
//!
//! This module should own the SPI thermocouple polling state machine. It should
//! update `SensorDataService` only; it should not format UART responses or drive
//! the status LED directly.

use crate::{
    events::AppEvent,
    plugins::{AppContext, Plugin},
};

pub struct ThermocouplePlugin {
    next_channel: usize,
}

impl ThermocouplePlugin {
    pub const fn new() -> Self {
        Self { next_channel: 0 }
    }
}

impl Plugin for ThermocouplePlugin {
    fn poll(&mut self, context: &mut AppContext<'_>) {
        // Future SPI work belongs here:
        // - select one of four chip-select pins
        // - perform a non-blocking or short SPI transaction
        // - decode the selected thermocouple IC's fault/status bits
        // - call context.sensor_data.set_thermocouple(...)
        // - push SensorUpdated or SensorFault events as appropriate
        self.next_channel =
            (self.next_channel + 1) % crate::services::sensor_data::THERMOCOUPLE_COUNT;

        if context.sensor_data.has_fault() {
            context.events.push(AppEvent::SensorFault);
        }
    }
}
