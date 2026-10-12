//! Firmware plugins.
//!
//! A plugin is a statically linked subsystem with a narrow responsibility and a
//! non-blocking `poll` method. This gives the project pseudo-RTOS organization
//! without adding an RTOS or changing HAL ecosystems.

use crate::{
    events::EventQueue,
    services::{sensor_data::SensorDataService, status::StatusService},
};

pub mod debug_console;
pub mod reporter;
pub mod status_led;
pub mod thermocouples;
pub mod uart;

pub struct AppContext<'a> {
    pub events: &'a mut EventQueue,
    pub sensor_data: &'a mut SensorDataService,
    pub status: &'a mut StatusService,
}

/// Tracks a plugin's cooperative poll cadence.
///
/// `interval_ms == 0` means the plugin should run every time it is polled.
/// Otherwise, `should_run` returns `true` only after the configured interval has
/// elapsed since the plugin last ran.
pub struct PollTimer {
    interval_ms: u32,
    last_run_ms: Option<u64>,
}

impl PollTimer {
    pub const fn new(interval_ms: u32) -> Self {
        Self {
            interval_ms,
            last_run_ms: None,
        }
    }

    pub fn should_run(&mut self, now_ms: u64) -> bool {
        if self.interval_ms == 0 {
            return true;
        }

        if let Some(last_run_ms) = self.last_run_ms {
            if now_ms.saturating_sub(last_run_ms) < u64::from(self.interval_ms) {
                return false;
            }
        }

        self.last_run_ms = Some(now_ms);
        true
    }
}

pub trait Plugin {
    /// Run one small unit of work.
    ///
    /// Implementations must not block for long periods. If a plugin needs to do
    /// slow work, split it into states and advance one state per call. Plugins
    /// that should not run every loop should store a `PollTimer` and return
    /// early when `now_ms` is before their next interval.
    fn poll(&mut self, context: &mut AppContext<'_>, now_ms: u64);
}
