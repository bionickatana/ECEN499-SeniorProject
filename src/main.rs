//! Firmware entry point.
//!
//! Keep this file intentionally small. Hardware setup belongs in `board.rs`,
//! application scheduling belongs in `app.rs`, and subsystem behavior belongs
//! in `plugins/`.

#![no_std]
#![no_main]

mod app;
mod board;
mod config;
mod events;
mod plugins;
mod services;

use panic_halt as _;

use rp235x_hal as hal;

/// Tell the RP2350 Boot ROM about our application.
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[hal::entry]
fn main() -> ! {
    let board = board::Board::init();
    let app = app::App::new(board);

    app.run()
}
