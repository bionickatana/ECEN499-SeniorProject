//! Application composition and cooperative polling.
//!
//! This is the pseudo-RTOS layer. It owns the plugins, polls each one every
//! loop, and provides shared services through `AppContext`. Plugins own their
//! poll cadence and return early when called before their next interval.

use rp235x_hal::timer::{CopyableTimer0, Timer};

use crate::{
    board::Board,
    events::{AppEvent, EventQueue},
    plugins::{
        AppContext, Plugin, debug_console::DebugConsolePlugin, reporter::ReporterPlugin,
        status_led::StatusLedPlugin, thermocouples::ThermocouplePlugin, uart::UartPlugin,
    },
    services::{
        sensor_data::SensorDataService,
        status::{StatusService, SystemStatus},
    },
};

pub struct App {
    timer: Timer<CopyableTimer0>,
    events: EventQueue,
    sensor_data: SensorDataService,
    status: StatusService,
    status_led: StatusLedPlugin,
    uart: UartPlugin,
    thermocouples: ThermocouplePlugin,
    reporter: ReporterPlugin,
    debug_console: DebugConsolePlugin,
}

impl App {
    pub fn new(board: Board) -> Self {
        let Board {
            timer,
            status_led,
            thermocouple,
            debug_console,
        } = board;
        let status_led = StatusLedPlugin::new(status_led);

        Self {
            timer,
            events: EventQueue::new(),
            sensor_data: SensorDataService::new(),
            status: StatusService::new(),
            status_led,
            uart: UartPlugin::new(),
            thermocouples: ThermocouplePlugin::new(thermocouple),
            reporter: ReporterPlugin::new(),
            debug_console: DebugConsolePlugin::new(debug_console),
        }
    }

    pub fn run(mut self) -> ! {
        self.events.push(AppEvent::BootComplete);
        self.status.set(SystemStatus::Operating);

        loop {
            self.poll_once();
        }
    }

    fn poll_once(&mut self) {
        let now_ms = self.now_ms();

        self.status.poll(now_ms);

        {
            let mut context = AppContext {
                events: &mut self.events,
                sensor_data: &mut self.sensor_data,
                status: &mut self.status,
            };

            self.uart.poll(&mut context, now_ms);
            self.debug_console.poll_usb();
            self.thermocouples.poll(&mut context, now_ms);
            self.debug_console.poll(&mut context, now_ms);
            self.reporter.poll(&mut context, now_ms);

            if context.events.contains(AppEvent::UartActivity) {
                context.status.mark_uart_activity(now_ms);
            }

            if context.events.contains(AppEvent::SensorFault)
                || context.events.contains(AppEvent::InternalError)
            {
                context.status.set(SystemStatus::Error);
            }

            self.status_led.poll(&mut context, now_ms);
        }

        self.events.clear();
    }

    fn now_ms(&self) -> u64 {
        self.timer.get_counter().ticks() / 1_000
    }
}
