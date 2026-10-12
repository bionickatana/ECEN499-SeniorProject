//! Reporting plugin placeholder.
//!
//! This module bridges requested reports to UART output. It should read shared
//! sensor data and build responses, but it should not perform SPI reads itself.

use crate::{
    config,
    events::AppEvent,
    plugins::{AppContext, Plugin, PollTimer},
};

pub struct ReporterPlugin {
    poll_timer: PollTimer,
}

impl ReporterPlugin {
    pub const fn new() -> Self {
        Self {
            poll_timer: PollTimer::new(config::REPORTER_POLL_INTERVAL_MS),
        }
    }
}

impl Plugin for ReporterPlugin {
    fn poll(&mut self, context: &mut AppContext<'_>, now_ms: u64) {
        if !self.poll_timer.should_run(now_ms) {
            return;
        }

        if !context.events.contains(AppEvent::ReportRequested) {
            return;
        }

        // Future reporting work belongs here:
        // - read context.sensor_data.thermocouples()
        // - encode the response in the UART protocol chosen by the UART owner
        // - hand bytes to the UART plugin or a shared TX service
        let _ = context.sensor_data.thermocouples();
    }
}
