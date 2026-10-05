//! # WS2812 Rainbow Example
//!
//! Drives the onboard WS2812 NeoPixel on the Waveshare RP2350-Zero.
//!
//! The onboard WS2812 data input is connected to GPIO16.
//!
//! This implementation uses only rp235x-hal 0.4.0 and the PIO peripheral.
//! It does not depend on ws2812-rp235x.
//!
//! WS2812 timing:
//!   - 800 kHz
//!   - 1.25 us per bit
//!   - GRB byte order
//!   - MSB first

#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
use panic_halt as _;

use rp235x_hal as hal;

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    pio::{PIOBuilder, PIOExt, ShiftDirection},
    timer::Timer,
    Watchdog,
    Sio,
};

/// Waveshare RP2350-Zero crystal frequency.
const XTAL_FREQ_HZ: u32 = 12_000_000;

/// Onboard WS2812 data pin.
const NEOPIXEL_PIN: u8 = 16;

/// WS2812 PIO timing.
///
/// The standard WS2812 PIO program uses 10 PIO cycles per bit.
///
/// At 8 MHz:
///
///     10 cycles / 8 MHz = 1.25 us
///
///     1 / 1.25 us = 800 kHz
const WS2812_PIO_FREQ_HZ: u32 = 8_000_000;

/// Tell the RP2350 Boot ROM about our application.
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef =
    hal::block::ImageDef::secure_exe();

#[hal::entry]
fn main() -> ! {
    // ---------------------------------------------------------------------
    // Peripheral setup
    // ---------------------------------------------------------------------

    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = Timer::new_timer0(
        pac.TIMER0,
        &mut pac.RESETS,
        &clocks,
    );

    let sio = Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // ---------------------------------------------------------------------
    // IMPORTANT:
    //
    // GPIO16 must be switched from SIO to PIO0.
    //
    // sm.set_pindirs() only tells the PIO state machine that the pin is
    // an output. It does NOT change the GPIO function mux.
    // ---------------------------------------------------------------------

    let neopixel_pin =
        pins.gpio16.into_function::<hal::gpio::FunctionPio0>();

    // Get the actual GPIO number for use by PIOBuilder.
    let neopixel_pin_id = neopixel_pin.id().num;

    // ---------------------------------------------------------------------
    // PIO setup
    // ---------------------------------------------------------------------

    let (mut pio, sm0, _, _, _) =
        pac.PIO0.split(&mut pac.RESETS);

    // Standard WS2812 PIO program.
    //
    // T1 = 3
    // T2 = 3
    // T3 = 4
    //
    // Total = 10 PIO cycles per bit.
    //
    // The side-set pin is:
    //
    //     LOW for T3 cycles
    //     HIGH for T1 + T2 cycles for a '1'
    //     HIGH for T1 cycles for a '0'
    //     LOW for the remaining T2 cycles
    //
    // This is the same basic timing arrangement used by Raspberry Pi's
    // official WS2812 PIO example.
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

    // Install the program into PIO0 instruction memory.
    let installed = pio.install(&program).unwrap();

    // ---------------------------------------------------------------------
    // Calculate the PIO clock divider.
    // ---------------------------------------------------------------------
    //
    // The state machine executes 10 cycles for every WS2812 bit.
    //
    // We want:
    //
    //     800,000 bits/sec
    //
    // therefore:
    //
    //     8,000,000 PIO cycles/sec
    //
    // The divider is:
    //
    //     system_clock / 8 MHz
    //
    // Calculating this from the actual clock is safer than assuming that
    // the RP2350 is always running at exactly 150 MHz.

    let system_clock_hz =
        clocks.system_clock.freq().to_Hz();

    let divisor =
        system_clock_hz as f32 / WS2812_PIO_FREQ_HZ as f32;

    let divisor_int = divisor as u16;

    let divisor_frac =
        ((divisor - divisor_int as f32) * 256.0) as u8;

    // ---------------------------------------------------------------------
    // Configure the PIO state machine.
    // ---------------------------------------------------------------------

    let (mut sm, _rx, mut tx) =
        PIOBuilder::from_installed_program(installed)

            // The WS2812 data line is controlled by side-set.
            .side_set_pin_base(neopixel_pin_id)

            // Shift the most-significant bit out first.
            .out_shift_direction(ShiftDirection::Left)

            // Pull a new 32-bit word after 24 bits have been shifted.
            .autopull(true)
            .pull_threshold(24)

            // 8 MHz PIO clock.
            .clock_divisor_fixed_point(
                divisor_int,
                divisor_frac,
            )

            .build(sm0);

    // Tell the PIO state machine that GPIO16 is an output.
    sm.set_pindirs([(
        neopixel_pin_id,
        hal::pio::PinDir::Output,
    )]);

    // Start PIO0 state machine 0.
    let _sm = sm.start();

    // ---------------------------------------------------------------------
    // Rainbow animation
    // ---------------------------------------------------------------------

    let mut n: u8 = 128;

    loop {
        let (r, g, b) = wheel(n);

        // Match the original example's brightness of 32/255.
        let r = scale_brightness(r, 32);
        let g = scale_brightness(g, 32);
        let b = scale_brightness(b, 32);

        // WS2812 expects GRB.
        //
        // We shift left, so the 24 useful bits are placed in bits
        // 31..8 of the FIFO word.
        //
        // This gives:
        //
        //     G7 ... G0 R7 ... R0 B7 ... B0 00000000
        //
        // with the MSB transmitted first.
        let color =
            ((g as u32) << 24) |
            ((r as u32) << 16) |
            ((b as u32) << 8);

        // Put the pixel into the PIO TX FIFO.
        tx.write(color);

        // Next colour.
        n = n.wrapping_add(1);

        // Animation speed.
        timer.delay_ms(25);
    }
}

/// Scale an 8-bit colour value by an 8-bit brightness value.
fn scale_brightness(value: u8, brightness: u8) -> u8 {
    ((value as u16 * brightness as u16) / 255) as u8
}

/// Convert a number from 0..=255 into an RGB colour.
///
/// The colour transitions:
///
///     red -> blue -> green -> red
fn wheel(mut wheel_pos: u8) -> (u8, u8, u8) {
    wheel_pos = 255 - wheel_pos;

    if wheel_pos < 85 {
        // Red -> blue
        (
            255 - wheel_pos * 3,
            0,
            wheel_pos * 3,
        )
    } else if wheel_pos < 170 {
        // Blue -> green
        wheel_pos -= 85;

        (
            0,
            wheel_pos * 3,
            255 - wheel_pos * 3,
        )
    } else {
        // Green -> red
        wheel_pos -= 170;

        (
            wheel_pos * 3,
            255 - wheel_pos * 3,
            0,
        )
    }
}
