use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{BinaryColor, PixelColor, Rgb888, RgbColor},
    Pixel,
};
use std::cmp::max;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TriColor {
    White,
    Black,
    Red,
}

impl PixelColor for TriColor {
    type Raw = ();
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
        if p.r() > p.g() && p.r() > p.b() {
            TriColor::Red
        } else if max(max(p.r(), p.g()), p.b()) > u8::MAX / 2 {
            TriColor::White
        } else {
            TriColor::Black
        }
    }
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
    //TODO implement rotation
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
        }
    }
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> OriginDimensions
    for Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    fn size(&self) -> Size {
        Size::new(SIZE_V, SIZE_H)
    }
}

impl<const SIZE_V: u32, const SIZE_H: u32, const IMAGE_SIZE: usize> DrawTarget
    for Display<SIZE_V, SIZE_H, IMAGE_SIZE>
{
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(point, color) = pixel;
            if (point.x < 0)
                || (point.x >= SIZE_H as i32)
                || (point.y < 0)
                || point.y >= SIZE_V as i32
            {
                continue;
            }

            let mask: u8 = 1 << (7 - (point.x % 8));
            let index = point.y as usize * SIZE_H as usize / 8 + point.x as usize / 8;
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
