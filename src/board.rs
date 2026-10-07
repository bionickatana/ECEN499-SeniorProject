//! Board-specific hardware setup.
//!
//! This file is the boundary between generic application structure and the
//! RP2350/Waveshare hardware. Contributors changing pin assignments or adding
//! UART/SPI peripheral initialization should start here and use constants from
//! `config.rs`.

use rp235x_hal as hal;

use core::mem::MaybeUninit;

use embedded_hal::digital::OutputPin;
use hal::{
    Sio, Watchdog,
    clocks::{Clock, init_clocks_and_plls},
    fugit::RateExtU32,
    gpio::{
        FunctionSioOutput, FunctionSpi, Pin, PullDown,
        bank0::{Gpio0, Gpio1, Gpio2, Gpio3},
    },
    pac,
    pio::{PIOBuilder, PIOExt, ShiftDirection},
    spi::{Enabled, Spi},
    timer::{CopyableTimer0, Timer},
};

use crate::{config, plugins::status_led::Ws2812Hardware};

use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;

pub struct Board {
    pub timer: Timer<CopyableTimer0>,
    pub status_led: Ws2812Hardware,
    pub thermocouple: ThermocoupleHardware,
    pub debug_console: DebugConsoleHardware,
}

type UsbBus = hal::usb::UsbBus;

static mut USB_BUS_ALLOCATOR: MaybeUninit<UsbBusAllocator<UsbBus>> = MaybeUninit::uninit();

pub type ThermocoupleSpi = Spi<
    Enabled,
    pac::SPI0,
    (
        Pin<Gpio3, FunctionSpi, PullDown>,
        Pin<Gpio0, FunctionSpi, PullDown>,
        Pin<Gpio2, FunctionSpi, PullDown>,
    ),
    8,
>;

pub type ThermocoupleCs = Pin<Gpio1, FunctionSioOutput, PullDown>;

pub struct ThermocoupleHardware {
    pub spi: ThermocoupleSpi,
    pub cs: ThermocoupleCs,
}

pub struct DebugConsoleHardware {
    pub usb_dev: UsbDevice<'static, UsbBus>,
    pub serial: SerialPort<'static, UsbBus>,
}

impl Board {
    pub fn init() -> Self {
        let mut pac = pac::Peripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);

        let clocks = init_clocks_and_plls(
            config::XTAL_FREQ_HZ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .unwrap();

        let timer = Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);
        let sio = Sio::new(pac.SIO);

        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // The HAL type for GPIO16 is explicit, so changing this constant also
        // requires updating the pin passed to `init_status_led` below.
        let _ = config::NEOPIXEL_PIN;
        let status_led = init_status_led(pac.PIO0, &mut pac.RESETS, pins.gpio16, &clocks);

        let thermocouple = init_thermocouple_spi(
            pac.SPI0,
            &mut pac.RESETS,
            pins.gpio0,
            pins.gpio1,
            pins.gpio2,
            pins.gpio3,
            &clocks,
        );

        let debug_console =
            init_debug_console(pac.USB, pac.USB_DPRAM, clocks.usb_clock, &mut pac.RESETS);

        // UART is intentionally not initialized yet. When contributors implement
        // it, add its hardware resource struct to `Board` so the matching plugin
        // owns only the peripheral it needs.
        let _ = config::UART_BAUD;
        let _ = config::UART_TX_PIN;
        let _ = config::UART_RX_PIN;

        Self {
            timer,
            status_led,
            thermocouple,
            debug_console,
        }
    }
}

fn init_debug_console(
    usb: pac::USB,
    usb_dpram: pac::USB_DPRAM,
    usb_clock: hal::clocks::UsbClock,
    resets: &mut pac::RESETS,
) -> DebugConsoleHardware {
    let usb_bus = hal::usb::UsbBus::new(usb, usb_dpram, usb_clock, true, resets);

    // The USB device and CDC serial class borrow the bus allocator for the full
    // firmware lifetime. Board initialization runs once before the app starts.
    let usb_bus_allocator = unsafe {
        let allocator = core::ptr::addr_of_mut!(USB_BUS_ALLOCATOR);
        (*allocator).write(UsbBusAllocator::new(usb_bus));
        &*(*allocator).as_ptr()
    };

    let serial = SerialPort::new(usb_bus_allocator);
    let usb_dev = UsbDeviceBuilder::new(usb_bus_allocator, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("ECEN499")
            .product("Thermocouple Debug Console")
            .serial_number("sw_demo")])
        .unwrap()
        .max_packet_size_0(64)
        .unwrap()
        .device_class(2)
        .build();

    DebugConsoleHardware { usb_dev, serial }
}

fn init_thermocouple_spi(
    spi0: pac::SPI0,
    resets: &mut pac::RESETS,
    miso_pin: Pin<hal::gpio::bank0::Gpio0, hal::gpio::FunctionNull, PullDown>,
    cs_pin: Pin<hal::gpio::bank0::Gpio1, hal::gpio::FunctionNull, PullDown>,
    sck_pin: Pin<hal::gpio::bank0::Gpio2, hal::gpio::FunctionNull, PullDown>,
    mosi_pin: Pin<hal::gpio::bank0::Gpio3, hal::gpio::FunctionNull, PullDown>,
    clocks: &hal::clocks::ClocksManager,
) -> ThermocoupleHardware {
    let _ = config::SPI_MISO_PIN;
    let _ = config::THERMOCOUPLE_CS_PINS[0];
    let _ = config::SPI_SCK_PIN;
    let _ = config::SPI_MOSI_PIN;

    let mosi = mosi_pin.into_function::<FunctionSpi>();
    let miso = miso_pin.into_function::<FunctionSpi>();
    let sck = sck_pin.into_function::<FunctionSpi>();
    let spi = Spi::<_, _, _, 8>::new(spi0, (mosi, miso, sck)).init(
        resets,
        clocks.peripheral_clock.freq(),
        config::SPI_BAUD_HZ.Hz(),
        embedded_hal::spi::MODE_0,
    );

    let mut cs = cs_pin.into_function::<FunctionSioOutput>();
    let _ = cs.set_high();

    ThermocoupleHardware { spi, cs }
}

fn init_status_led(
    pio0: pac::PIO0,
    resets: &mut pac::RESETS,
    neopixel_pin: hal::gpio::Pin<
        hal::gpio::bank0::Gpio16,
        hal::gpio::FunctionNull,
        hal::gpio::PullDown,
    >,
    clocks: &hal::clocks::ClocksManager,
) -> Ws2812Hardware {
    // GPIO16 must be switched from SIO to PIO0. Setting PIO pin directions is
    // not enough; the GPIO function mux must also point at PIO0.
    let neopixel_pin = neopixel_pin.into_function::<hal::gpio::FunctionPio0>();
    let neopixel_pin_id = neopixel_pin.id().num;

    let (mut pio, sm0, _, _, _) = pio0.split(resets);

    // Standard WS2812 program. It uses 10 PIO cycles per bit and transmits GRB
    // bytes MSB-first at the configured 800 kHz data rate.
    let program = pio::pio_asm!(
        ".side_set 1",
        ".wrap_target",
        "bitloop:",
        "    out x, 1       side 0 [3]",
        "    jmp !x, do_zero side 1 [2]",
        "    jmp bitloop     side 1 [2]",
        "do_zero:",
        "    nop             side 0 [2]",
        ".wrap"
    )
    .program;

    let installed = pio.install(&program).unwrap();
    let system_clock_hz = clocks.system_clock.freq().to_Hz();
    let divisor = system_clock_hz as f32 / config::WS2812_PIO_FREQ_HZ as f32;
    let divisor_int = divisor as u16;
    let divisor_frac = ((divisor - divisor_int as f32) * 256.0) as u8;

    let (mut sm, _rx, tx) = PIOBuilder::from_installed_program(installed)
        .side_set_pin_base(neopixel_pin_id)
        .out_shift_direction(ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(24)
        .clock_divisor_fixed_point(divisor_int, divisor_frac)
        .build(sm0);

    sm.set_pindirs([(neopixel_pin_id, hal::pio::PinDir::Output)]);
    let sm = sm.start();

    Ws2812Hardware::new(tx, sm)
}
