//! Application events shared between plugins.
//!
//! Events describe what happened without deciding who should handle it. This is
//! how plugins stay isolated: UART can announce a report request, the reporter
//! can consume it, and the status LED can react to activity without those
//! modules directly depending on each other.

#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum AppEvent {
    BootComplete,
    UartActivity,
    ReportRequested,
    SensorUpdated,
    SensorFault,
    InternalError,
}

/// Small fixed event queue for this cooperative firmware.
///
/// This intentionally avoids heap allocation. If the application grows beyond a
/// few event types, replace this with a tested fixed-capacity queue crate rather
/// than adding ad-hoc buffering logic in plugins.
pub struct EventQueue {
    events: [Option<AppEvent>; Self::CAPACITY],
}

impl EventQueue {
    const CAPACITY: usize = 8;

    pub const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
        }
    }

    pub fn push(&mut self, event: AppEvent) {
        for slot in self.events.iter_mut() {
            if slot.is_none() {
                *slot = Some(event);
                return;
            }
        }
    }

    pub fn contains(&self, event: AppEvent) -> bool {
        self.events.iter().any(|queued| *queued == Some(event))
    }

    pub fn clear(&mut self) {
        for slot in self.events.iter_mut() {
            *slot = None;
        }
    }
}
