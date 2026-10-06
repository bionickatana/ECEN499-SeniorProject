//! Board-specific hardware setup.
//!
//! This file is the boundary between generic application structure and the
//! RP2350/Waveshare hardware. Contributors changing pin assignments or adding
//! UART/SPI peripheral initialization should start here and use constants from
//! `config.rs`.

use rp235x_hal as hal;

use hal::{
    Sio, Watchdog,
    clocks::{Clock, init_clocks_and_plls},
    pac,
    pio::{PIOBuilder, PIOExt, ShiftDirection},
    timer::{CopyableTimer0, Timer},
};

use crate::{config, plugins::status_led::Ws2812Hardware};

pub struct Board {
    pub timer: Timer<CopyableTimer0>,
    pub status_led: Ws2812Hardware,
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

        // UART and SPI are intentionally not initialized yet. When contributors
        // implement them, add their hardware resource structs to `Board` so the
        // matching plugin owns only the peripheral it needs.
        let _ = config::UART_BAUD;
        let _ = config::UART_TX_PIN;
        let _ = config::UART_RX_PIN;
        let _ = config::SPI_SCK_PIN;
        let _ = config::SPI_MOSI_PIN;
        let _ = config::SPI_MISO_PIN;
        let _ = config::THERMOCOUPLE_CS_PINS;

        Self { timer, status_led }
    }
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
