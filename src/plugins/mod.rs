//! Firmware plugins.
//!
//! A plugin is a statically linked subsystem with a narrow responsibility and a
//! non-blocking `poll` method. This gives the project pseudo-RTOS organization
//! without adding an RTOS or changing HAL ecosystems.

use crate::{
    events::EventQueue,
    services::{sensor_data::SensorDataService, status::StatusService},
};

pub mod reporter;
pub mod status_led;
pub mod thermocouples;
pub mod uart;

pub struct AppContext<'a> {
    pub events: &'a mut EventQueue,
    pub sensor_data: &'a mut SensorDataService,
    pub status: &'a mut StatusService,
}

pub trait Plugin {
    /// Run one small unit of work.
    ///
    /// Implementations must not block for long periods. If a plugin needs to do
    /// slow work, split it into states and advance one state per call.
    fn poll(&mut self, context: &mut AppContext<'_>);
}
