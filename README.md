# Driver for Spectra E-Paper Displays from Pervasive Displays Inc
[![Crates.io](https://img.shields.io/crates/v/epd-spectra.svg)](https://crates.io/crates/epd-spectra)
[![Docs.rs](https://docs.rs/epd-spectra/badge.svg)](https://docs.rs/epd-spectra)

This library contains a driver written in Rust for the Spectra tri-colour (white, black, red) e-paper displays from [Pervasive Displays Inc](https://github.com/PervasiveDisplays). The displays are often found in SES Imagotag electronic price labels. Technical details of the price labels can be found for example [here](https://github.com/andrei-tatar/imagotag-hack).

The C++ driver can be found [here](https://github.com/PervasiveDisplays/EPD_Driver_GU_small).

This library is tested with the 2.66 inch display and the [EXT3-1 extension kit](https://docs.pervasivedisplays.com/epd-usage/development-kits/ext3-1) from Pervasive Displays on a Raspberry Pi Zero with std support and on a STM32 Nucleo board with no_std. See the examples folder to get started.

