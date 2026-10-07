//! Native USB CDC debug console.
//!
//! This plugin is separate from the UART plugin. UART remains reserved for the
//! external device protocol; this console is only for host-side debug output via
//! the board's USB-C connector.

use core::fmt::Write;

use heapless::String;

use crate::{
    board::DebugConsoleHardware,
    plugins::{AppContext, Plugin},
    services::sensor_data::ThermocoupleReading,
};

pub struct DebugConsolePlugin {
    hardware: DebugConsoleHardware,
}

impl DebugConsolePlugin {
    pub const fn new(hardware: DebugConsoleHardware) -> Self {
        Self { hardware }
    }

    pub fn poll_usb(&mut self) {
        let _ = self.hardware.usb_dev.poll(&mut [&mut self.hardware.serial]);
    }

    fn write_reading(&mut self, reading: ThermocoupleReading) {
        let mut line: String<64> = String::new();

        if reading.fault {
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
    fn poll(&mut self, context: &mut AppContext<'_>) {
        self.poll_usb();
        let reading = context.sensor_data.thermocouples()[0];
        self.write_reading(reading);
    }
}
