//! This library contains a driver written in Rust for the Spectra
//! tri-colour (white, black, red) e-paper displays from
//! [Pervasive Displays Inc](https://github.com/PervasiveDisplays).
//!
//! See the examples folder to get started.
#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod driver;
pub mod graphics;

pub use driver::*;
pub use graphics::*;
