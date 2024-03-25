//! Generic SPI driver for all EPDs

use core::marker::PhantomData;
use embedded_hal::{delay::DelayNs, digital::InputPin, digital::OutputPin, spi::SpiDevice};

use crate::DisplayBuffer;

enum Command {
    Psr = 0x00,
    PowerOff = 0x02,
    PowerOn = 0x04,
    BufferBlack = 0x10,
    Refresh = 0x12,
    BufferRed = 0x13,
    ActiveTemperature = 0xe0,
    InputTemperature = 0xe5,
}

/// Config register data for sizes other than 4.2"
const REG_DATA_SOFT_RESET: &[u8] = &[0x0e];
const REG_DATA_INPUT_TEMP: &[u8] = &[0x19];
const REG_DATA_ACTIVE_TEMP: &[u8] = &[0x02];
const REG_DATA_PSR: &[u8] = &[0xcf, 0x8d];

// Sadly we cannot use #[from] more than once.
// See here for similiar problem: https://stackoverflow.com/questions/37347311/how-is-there-a-conflicting-implementation-of-from-when-using-a-generic-type

#[cfg(feature = "std")]
#[derive(thiserror::Error, Debug)]
pub enum Error<SpiError, DcError, RstError> {
    #[error("SPI error: {0}")]
    Spi(#[source] SpiError),
    #[error("Error with GPIO 'DC': {0}")]
    GpioDc(#[source] DcError),
    #[error("Error with GPIO 'RESET': {0}")]
    GpioRst(#[source] RstError),
}

#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub enum Error<SpiError, DcError, RstError> {
    Spi(SpiError),
    GpioDc(DcError),
    GpioRst(RstError),
}

type EpdError<SPI, DC, RST> = Error<
    <SPI as embedded_hal::spi::ErrorType>::Error,
    <DC as embedded_hal::digital::ErrorType>::Error,
    <RST as embedded_hal::digital::ErrorType>::Error,
>;

/// Actual driver for e-paper display
pub struct Epd<SPI, BUSY, DC, RST, DELAY> {
    /// busy pin, active low
    busy: BUSY,
    /// Data/Command control pin (data: high, command: low)
    dc: DC,
    /// reset pin, active low
    rst: RST,
    /// chunk size used for SPI writes (0: no chunks)
    spi_chunk_size: usize,
    _spi: PhantomData<SPI>,
    _delay: PhantomData<DELAY>,
}

#[allow(clippy::missing_errors_doc)]
impl<SPI, BUSY, DC, RST, DELAY> Epd<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    /// Create a new e-paper driver and run initialization sequence.
    /// `spi_chunk_size` determines the data chunk size for SPI writes, 0 means no chunks.
    /// E.g. Linux has a default buffer size of 4096. So `spi_chunk_size` must be equal to or smaller than 4096.
    pub fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        spi_chunk_size: usize,
    ) -> Result<Self, EpdError<SPI, DC, RST>> {
        let mut epd = Self {
            busy,
            dc,
            rst,
            spi_chunk_size,
            _spi: PhantomData,
            _delay: PhantomData,
        };
        epd.init(spi, delay)?;
        Ok(epd)
    }

    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), EpdError<SPI, DC, RST>> {
        self.dc.set_high().map_err(Error::GpioDc)?;
        self.reset(delay)?;
        self.soft_reset(spi)?;
        self.send_data(spi, Command::InputTemperature, REG_DATA_INPUT_TEMP)?;
        self.send_data(spi, Command::ActiveTemperature, REG_DATA_ACTIVE_TEMP)?;
        self.send_data(spi, Command::Psr, REG_DATA_PSR)?;
        Ok(())
    }

    /// Show display on e-paper
    pub fn update(
        &mut self,
        display: &impl DisplayBuffer,
        spi: &mut SPI,
    ) -> Result<(), EpdError<SPI, DC, RST>> {
        self.send_data(spi, Command::BufferBlack, display.get_buffer_black())?;
        self.send_data(spi, Command::BufferRed, display.get_buffer_red())?;
        self.power_on(spi)?;
        self.display_refresh(spi)?;
        Ok(())
    }

    pub fn reset(&mut self, delay: &mut DELAY) -> Result<(), EpdError<SPI, DC, RST>> {
        delay.delay_ms(1);
        self.rst.set_high().map_err(Error::GpioRst)?;
        delay.delay_ms(5);
        self.rst.set_low().map_err(Error::GpioRst)?;
        delay.delay_ms(10);
        self.rst.set_high().map_err(Error::GpioRst)?;
        delay.delay_ms(5);
        Ok(())
    }

    pub fn power_on(&mut self, spi: &mut SPI) -> Result<(), EpdError<SPI, DC, RST>> {
        self.send_data(spi, Command::PowerOn, &[0x0])?;
        self.wait_busy();
        Ok(())
    }

    pub fn power_off(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), EpdError<SPI, DC, RST>> {
        self.send_data(spi, Command::PowerOff, &[0x0])?;
        self.wait_busy();
        self.dc.set_low().map_err(Error::GpioDc)?;
        delay.delay_ms(150);
        self.rst.set_low().map_err(Error::GpioRst)?;
        Ok(())
    }

    fn send_data(
        &mut self,
        spi: &mut SPI,
        cmd: Command,
        data: &[u8],
    ) -> Result<(), EpdError<SPI, DC, RST>> {
        self.dc.set_low().map_err(Error::GpioDc)?;
        self.write(spi, &[cmd as u8])?;
        self.dc.set_high().map_err(Error::GpioDc)?;
        self.write(spi, data)?;
        Ok(())
    }

    fn write(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), EpdError<SPI, DC, RST>> {
        if self.spi_chunk_size > 0 {
            for chunk in data.chunks(self.spi_chunk_size) {
                spi.write(chunk).map_err(Error::Spi)?;
            }
        } else {
            spi.write(data).map_err(Error::Spi)?;
        }
        Ok(())
    }

    fn soft_reset(&mut self, spi: &mut SPI) -> Result<(), EpdError<SPI, DC, RST>> {
        self.send_data(spi, Command::Psr, REG_DATA_SOFT_RESET)?;
        self.wait_busy();
        Ok(())
    }

    fn display_refresh(&mut self, spi: &mut SPI) -> Result<(), EpdError<SPI, DC, RST>> {
        self.send_data(spi, Command::Refresh, &[0x0])?;
        self.wait_busy();
        Ok(())
    }

    fn wait_busy(&mut self) {
        while self.busy.is_low().unwrap() {}
    }
}

/// SPI mode needed for EPD driver
/// Mode0: CPOL 0, CPHA 0
pub const SPI_MODE: embedded_hal::spi::Mode = embedded_hal::spi::Mode {
    phase: embedded_hal::spi::Phase::CaptureOnFirstTransition,
    polarity: embedded_hal::spi::Polarity::IdleLow,
};
