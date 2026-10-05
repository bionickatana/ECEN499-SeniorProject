//! Reporting plugin placeholder.
//!
//! This module bridges requested reports to UART output. It should read shared
//! sensor data and build responses, but it should not perform SPI reads itself.

use crate::{
    events::AppEvent,
    plugins::{AppContext, Plugin},
};

pub struct ReporterPlugin;

impl ReporterPlugin {
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for ReporterPlugin {
    fn poll(&mut self, context: &mut AppContext<'_>) {
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
