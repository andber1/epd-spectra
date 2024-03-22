use embedded_hal::{delay::DelayNs, digital::InputPin, digital::OutputPin, spi::SpiDevice};
use std::marker::PhantomData;

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
#[derive(thiserror::Error, Debug)]
pub enum Error<SpiError, DcError, RstError> {
    #[error("SPI error: {0}")]
    Spi(#[from] SpiError),
    #[error("Error with GPIO 'DC': {0}")]
    GpioDc(#[source] DcError),
    #[error("Error with GPIO 'RESET': {0}")]
    GpioRst(#[source] RstError),
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
    _spi: PhantomData<SPI>,
    _delay: PhantomData<DELAY>,
}

impl<SPI, BUSY, DC, RST, DELAY> Epd<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    /// Create a new e-paper driver and run initialization sequence
    pub fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, EpdError<SPI, DC, RST>> {
        let mut epd = Self {
            busy,
            dc,
            rst,
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
        //TODO Implement single byte write or divide buffer into chunks. SPI hardware buffer might not be large enough for whole data.
        spi.write(&[cmd as u8])?;
        self.dc.set_high().map_err(Error::GpioDc)?;
        spi.write(data)?;
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
