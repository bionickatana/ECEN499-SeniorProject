//! System status model used by the status LED and other plugins.
//!
//! Put status policy here. The NeoPixel driver should only render the selected
//! pattern; it should not know why the application entered a status state.

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 180, 0);

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn scaled(self, brightness: u8) -> Self {
        Self {
            red: scale_u8(self.red, brightness),
            green: scale_u8(self.green, brightness),
            blue: scale_u8(self.blue, brightness),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum LedPattern {
    Off,
    Solid(Color),
    Breathing(Color),
    Flashing(Color),
    Pulse(Color),
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum SystemStatus {
    Booting,
    Operating,
    Error,
    UartActivity,
    Custom(LedPattern),
}

impl SystemStatus {
    pub fn led_pattern(self) -> LedPattern {
        match self {
            SystemStatus::Booting => LedPattern::Breathing(Color::YELLOW),
            SystemStatus::Operating => LedPattern::Breathing(Color::GREEN),
            SystemStatus::Error => LedPattern::Flashing(Color::RED),
            SystemStatus::UartActivity => LedPattern::Pulse(Color::BLUE),
            SystemStatus::Custom(pattern) => pattern,
        }
    }
}

pub struct StatusService {
    current: SystemStatus,
    previous: SystemStatus,
    activity_ticks_remaining: u8,
}

impl StatusService {
    pub const fn new() -> Self {
        Self {
            current: SystemStatus::Booting,
            previous: SystemStatus::Booting,
            activity_ticks_remaining: 0,
        }
    }

    pub fn current(&self) -> SystemStatus {
        self.current
    }

    pub fn set(&mut self, status: SystemStatus) {
        self.previous = self.current;
        self.current = status;
    }

    pub fn mark_uart_activity(&mut self) {
        if self.current != SystemStatus::UartActivity {
            self.previous = self.current;
        }

        self.current = SystemStatus::UartActivity;
        self.activity_ticks_remaining = 30;
    }

    pub fn poll(&mut self) {
        if self.current != SystemStatus::UartActivity {
            return;
        }

        self.activity_ticks_remaining = self.activity_ticks_remaining.saturating_sub(1);

        if self.activity_ticks_remaining == 0 {
            self.current = self.previous;
        }
    }
}

fn scale_u8(value: u8, brightness: u8) -> u8 {
    ((value as u16 * brightness as u16) / 255) as u8
}
