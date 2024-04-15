//! Specific display buffers for each EPDs and `embedded_graphics` related implementations

use core::cmp::{max, min};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{
        raw::{RawData, RawU2},
        BinaryColor, PixelColor, Rgb888, RgbColor,
    },
    Pixel,
};

/// Colors supported by the e-paper displays
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TriColor {
    #[default]
    White,
    Black,
    Red,
}

impl PixelColor for TriColor {
    type Raw = RawU2;
}

impl From<RawU2> for TriColor {
    fn from(data: RawU2) -> Self {
        let data = data.into_inner();
        if data & 0b01 != 0 {
            TriColor::Black
        } else if data & 0b10 != 0 {
            TriColor::Red
        } else {
            TriColor::White
        }
    }
}

impl From<BinaryColor> for TriColor {
    fn from(b: BinaryColor) -> TriColor {
        match b {
            BinaryColor::On => Self::Black,
            BinaryColor::Off => Self::White,
        }
    }
}

impl From<TriColor> for Rgb888 {
    fn from(b: TriColor) -> Self {
        match b {
            TriColor::White => Self::new(u8::MAX, u8::MAX, u8::MAX),
            TriColor::Black => Self::new(0, 0, 0),
            TriColor::Red => Self::new(u8::MAX, 0, 0),
        }
    }
}

impl From<Rgb888> for TriColor {
    fn from(p: Rgb888) -> TriColor {
        let min = min(min(p.r(), p.g()), p.b());
        let max = max(max(p.r(), p.g()), p.b());
        let chroma = max - min;
        let brightness = max;
        if chroma > u8::MAX / 3 && p.r() > p.g() && p.r() > p.b() {
            TriColor::Red
        } else if brightness > u8::MAX / 2 {
            TriColor::White
        } else {
            TriColor::Black
        }
    }
}

/// Display rotation, only 90° increments supported
#[derive(Clone, Copy, Default)]
pub enum DisplayRotation {
    /// No rotation
    #[default]
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

pub trait DisplayBuffer {
    fn get_buffer_black(&self) -> &[u8];
    fn get_buffer_red(&self) -> &[u8];
}

/// Display buffer used for drawing with `embedded_graphics`.
/// The concrete types are dependent on the size.
/// Examples: `Display1in54`, `Display2in13`, ...
pub struct Display<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> {
    buffer_black: [u8; IMAGE_SIZE],
    buffer_red: [u8; IMAGE_SIZE],
    rotation: DisplayRotation,
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize>
    Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }
    #[must_use]
    pub fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> DisplayBuffer
    for Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    fn get_buffer_black(&self) -> &[u8] {
        &self.buffer_black
    }
    fn get_buffer_red(&self) -> &[u8] {
        &self.buffer_red
    }
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> Default
    for Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    fn default() -> Self {
        Self {
            buffer_black: [0; IMAGE_SIZE],
            buffer_red: [0; IMAGE_SIZE],
            rotation: DisplayRotation::default(),
        }
    }
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> OriginDimensions
    for Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    fn size(&self) -> Size {
        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => Size::new(SIZE_H, SIZE_V),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => Size::new(SIZE_V, SIZE_H),
        }
    }
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> DrawTarget
    for Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    type Color = TriColor;
    type Error = core::convert::Infallible;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(p, color) = pixel;

            let (x, y) = match self.rotation {
                DisplayRotation::Rotate0 => (p.x, p.y),
                DisplayRotation::Rotate90 => (SIZE_H as i32 - 1 - p.y, p.x),
                DisplayRotation::Rotate180 => (SIZE_H as i32 - 1 - p.x, SIZE_V as i32 - 1 - p.y),
                DisplayRotation::Rotate270 => (p.y, SIZE_V as i32 - 1 - p.x),
            };

            if (x < 0) || (x >= SIZE_H as i32) || (y < 0) || y >= SIZE_V as i32 {
                continue;
            }

            let mask: u8 = 1 << (7 - (x % 8));
            let index = y as usize * SIZE_H as usize / 8 + x as usize / 8;
            assert!(index < IMAGE_SIZE);

            match color {
                TriColor::White => {
                    self.buffer_black[index] &= !mask;
                    self.buffer_red[index] &= !mask;
                }
                TriColor::Black => {
                    self.buffer_black[index] |= mask;
                    self.buffer_red[index] &= !mask;
                }
                TriColor::Red => {
                    self.buffer_black[index] &= !mask;
                    self.buffer_red[index] |= mask;
                }
            }
        }
        Ok(())
    }
}

macro_rules! display_type {
    ($a:expr, $b:expr) => {
        Display<$a, $b, {$a * ($b / 8)}>
    };
}
pub type Display1in54 = display_type!(152, 152);
pub type Display2in13 = display_type!(212, 104);
pub type Display2in66 = display_type!(296, 152);
pub type Display2in71 = display_type!(264, 176);
pub type Display2in87 = display_type!(296, 128);
pub type Display3in70 = display_type!(416, 240);
pub type Display4in17 = display_type!(300, 400);
pub type Display4in37 = display_type!(480, 176);
