//! SPI thermocouple plugin.
//!
//! This module owns MAX6675 polling and writes the latest decoded reading into
//! `SensorDataService`. It does not format UART responses or drive the status
//! LED directly. Read cadence is controlled by the plugin's poll timer.

use embedded_hal::{digital::OutputPin, spi::SpiBus};

use crate::{
    board::ThermocoupleHardware,
    config,
    events::AppEvent,
    plugins::{AppContext, Plugin, PollTimer},
    services::sensor_data::ThermocoupleReading,
};

pub struct ThermocouplePlugin {
    hardware: ThermocoupleHardware,
    poll_timer: PollTimer,
}

impl ThermocouplePlugin {
    pub const fn new(hardware: ThermocoupleHardware) -> Self {
        Self {
            hardware,
            poll_timer: PollTimer::new(config::THERMOCOUPLE_POLL_INTERVAL_MS),
        }
    }

    fn read_channel_0(&mut self) -> ThermocoupleReading {
        let mut buffer = [0_u8; 2];

        let _ = self.hardware.cs.set_low();
        let transfer_result = self.hardware.spi.transfer_in_place(&mut buffer);
        let _ = self.hardware.cs.set_high();

        if transfer_result.is_err() {
            return ThermocoupleReading {
                temperature_celsius: None,
                fault: true,
            };
        }

        decode_max6675(buffer)
    }
}

impl Plugin for ThermocouplePlugin {
    fn poll(&mut self, context: &mut AppContext<'_>, now_ms: u64) {
        if !self.poll_timer.should_run(now_ms) {
            return;
        }

        let reading = self.read_channel_0();
        context.sensor_data.set_thermocouple(0, reading);

        if reading.fault {
            context.events.push(AppEvent::SensorFault);
        } else {
            context.events.push(AppEvent::SensorUpdated);
        }
    }
}

fn decode_max6675(buffer: [u8; 2]) -> ThermocoupleReading {
    let raw = u16::from_be_bytes(buffer);

    if raw & 0x0004 != 0 {
        return ThermocoupleReading {
            temperature_celsius: None,
            fault: true,
        };
    }

    let quarter_degree_count = (raw >> 3) & 0x0fff;
    let hundredths_celsius = u32::from(quarter_degree_count) * 25;

    ThermocoupleReading {
        temperature_celsius: Some(fixed_point::FixedPoint(hundredths_celsius)),
        fault: false,
    }
}
