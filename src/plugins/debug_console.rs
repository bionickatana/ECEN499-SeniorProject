//! Native USB CDC debug console.
//!
//! This plugin is separate from the UART plugin. UART remains reserved for the
//! external device protocol; this console is only for host-side debug output via
//! the board's USB-C connector. The app polls USB every loop and rate-limits
//! this plugin's full `poll()` with the plugin's poll timer.

use core::fmt::Write;

use heapless::String;

use crate::{
    board::DebugConsoleHardware,
    config,
    plugins::{AppContext, Plugin, PollTimer},
    services::sensor_data::ThermocoupleReading,
};

pub struct DebugConsolePlugin {
    hardware: DebugConsoleHardware,
    poll_timer: PollTimer,
}

impl DebugConsolePlugin {
    pub const fn new(hardware: DebugConsoleHardware) -> Self {
        Self {
            hardware,
            poll_timer: PollTimer::new(config::DEBUG_CONSOLE_REPORT_INTERVAL_MS),
        }
    }

    pub fn poll_usb(&mut self) {
        let _ = self.hardware.usb_dev.poll(&mut [&mut self.hardware.serial]);
    }

    fn write_reading(&mut self, reading: ThermocoupleReading){//, u8: idx) {
        let mut line: String<64> = String::new();

        if reading.fault {
            //let _ = writeln!(&mut line, "TC{idx}: FAULT\r");
            let _ = writeln!(&mut line, "TC0: FAULT\r");
        }
        if let Some(temperature) = reading.temperature_celsius {
            let raw = temperature.0;
            let integer = raw / 100;
            let decimal = raw % 100;
            let _ = writeln!(&mut line, "TC0: {integer}.{decimal:02} C\r");
        } else {
            let _ = writeln!(&mut line, "TC0: unavailable\r");
        }

        let _ = self.hardware.serial.write(line.as_bytes());
    }
}

impl Plugin for DebugConsolePlugin {
    fn poll(&mut self, context: &mut AppContext<'_>, now_ms: u64) {
        if !self.poll_timer.should_run(now_ms) {
            return;
        }

        self.poll_usb();
        let reading = context.sensor_data.thermocouples()[0];
        self.write_reading(reading);
//        let reading = context.sensor_data.thermocouples()[1];
//        self.write_reading(reading, 1);
//        let reading = context.sensor_data.thermocouples()[2];
//        self.write_reading(reading, 2);
//        let reading = context.sensor_data.thermocouples()[3];
//        self.write_reading(reading, 3);
    }
}
