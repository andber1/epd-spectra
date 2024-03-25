//! Simple "Hello World" example for a raspberry with a 2.66 inch e-paper display.
//!
//! Connections:
//!
//! | Raspberry      | EPD   |
//! |----------------|-------|
//! | GPIO 11 (SCLK) | SCK   |
//! | GPIO 10 (MOSI) | MOSI  |
//! | GPIO 8  (CE0)  | CS    |
//! | GPIO 24        | BUSY  |
//! | GPIO 25        | DC    |
//! | GPIO 17        | RESET |
//!
//! If you have another display size, simply replace `Display2in66` with your display.
//! You have to enable SPI (e.g. with raspi-config) and you have to execute the binary with sudo:
//! `cargo build --example raspberry --features="std" && sudo ./target/debug/examples/raspberry`

use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_10X20, MonoTextStyle},
    prelude::*,
    text::Text,
};
use epd_spectra::{Display2in66, Epd, TriColor};
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SimpleHalSpiDevice, SlaveSelect, Spi},
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // setup a display buffer
    let mut display = Display2in66::default();

    Text::new(
        "Hello",
        Point::new(10, 20),
        MonoTextStyle::new(&FONT_10X20, TriColor::Black),
    )
    .draw(&mut display)?;

    Text::new(
        "World",
        Point::new(30, 60),
        MonoTextStyle::new(&FONT_10X20, TriColor::Red),
    )
    .draw(&mut display)?;

    // setup all peripherals needed for EPD driver
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_000_000, Mode::Mode0)?;

    let mut spi_device = SimpleHalSpiDevice::new(spi);
    let gpio = Gpio::new()?;

    let busy = gpio.get(24)?.into_input();
    let dc = gpio.get(25)?.into_output();
    let rst = gpio.get(17)?.into_output();
    let mut delay = Delay {};

    let mut epd = Epd::new(&mut spi_device, busy, dc, rst, &mut delay, 4096)?;

    // show the display
    epd.update(&display, &mut spi_device)?;
    epd.power_off(&mut spi_device, &mut delay)?;

    Ok(())
}
