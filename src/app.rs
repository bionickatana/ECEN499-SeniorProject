//! Application composition and cooperative scheduling.
//!
//! This is the pseudo-RTOS layer. It owns the plugins, polls each one regularly,
//! and provides shared services through `AppContext`. New subsystem code should
//! usually be added as a plugin instead of expanding this file.

use embedded_hal::delay::DelayNs;
use rp235x_hal::timer::{CopyableTimer0, Timer};

use crate::{
    board::Board,
    config,
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
    thermocouple_poll_count: u32,
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
            thermocouple_poll_count: 0,
        }
    }

    pub fn run(mut self) -> ! {
        self.events.push(AppEvent::BootComplete);
        self.status.set(SystemStatus::Operating);

        loop {
            self.poll_once();
            self.timer.delay_ms(config::APP_LOOP_DELAY_MS);
        }
    }

    fn poll_once(&mut self) {
        self.status.poll();

        {
            let mut context = AppContext {
                events: &mut self.events,
                sensor_data: &mut self.sensor_data,
                status: &mut self.status,
            };

            self.uart.poll(&mut context);
            self.debug_console.poll_usb();
            self.thermocouple_poll_count += 1;
            let thermocouple_interval_polls =
                (config::THERMOCOUPLE_READ_INTERVAL_MS / config::APP_LOOP_DELAY_MS).max(1);
            if self.thermocouple_poll_count >= thermocouple_interval_polls {
                self.thermocouple_poll_count = 0;
                self.thermocouples.poll(&mut context);
                self.debug_console.poll(&mut context);
            }
            self.reporter.poll(&mut context);

            if context.events.contains(AppEvent::UartActivity) {
                context.status.mark_uart_activity();
            }

            if context.events.contains(AppEvent::SensorFault)
                || context.events.contains(AppEvent::InternalError)
            {
                context.status.set(SystemStatus::Error);
            }

            self.status_led.poll(&mut context);
        }

        self.events.clear();
    }
}
