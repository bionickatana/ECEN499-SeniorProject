//! Shared thermocouple readings.
//!
//! The thermocouple plugin owns the SPI reads and writes fresh values here. The
//! reporter plugin reads this service when UART asks for data. This prevents SPI
//! code and reporting/protocol code from being mixed together.

pub const THERMOCOUPLE_COUNT: usize = 4;

pub type TemperatureCelsius = fixed_point::FixedPoint<u32, 2>;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ThermocoupleReading {
    pub temperature_celsius: Option<TemperatureCelsius>,
    pub fault: bool,
}

impl ThermocoupleReading {
    pub const fn unavailable() -> Self {
        Self {
            temperature_celsius: None,
            fault: false,
        }
    }
}

pub struct SensorDataService {
    thermocouples: [ThermocoupleReading; THERMOCOUPLE_COUNT],
}

impl SensorDataService {
    pub const fn new() -> Self {
        Self {
            thermocouples: [ThermocoupleReading::unavailable(); THERMOCOUPLE_COUNT],
        }
    }

    pub fn thermocouples(&self) -> &[ThermocoupleReading; THERMOCOUPLE_COUNT] {
        &self.thermocouples
    }

    #[allow(dead_code)]
    pub fn set_thermocouple(&mut self, index: usize, reading: ThermocoupleReading) {
        if let Some(slot) = self.thermocouples.get_mut(index) {
            *slot = reading;
        }
    }

    pub fn has_fault(&self) -> bool {
        self.thermocouples.iter().any(|reading| reading.fault)
    }
}
