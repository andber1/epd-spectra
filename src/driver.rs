use embedded_hal::{delay::DelayNs, digital::InputPin, digital::OutputPin, spi::SpiDevice};
use std::{error::Error, marker::PhantomData};

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
    <BUSY as embedded_hal::digital::ErrorType>::Error: Error + 'static,
    <DC as embedded_hal::digital::ErrorType>::Error: Error + 'static,
    <RST as embedded_hal::digital::ErrorType>::Error: Error + 'static,
    <SPI as embedded_hal::spi::ErrorType>::Error: Error + 'static,
{
    /// Create a new e-paper driver and run initialization sequence
    pub fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, Box<dyn Error>> {
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

    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), Box<dyn Error>> {
        self.dc.set_high()?;
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
    ) -> Result<(), Box<dyn Error>> {
        self.send_data(spi, Command::BufferBlack, display.get_buffer_black())?;
        self.send_data(spi, Command::BufferRed, display.get_buffer_red())?;
        self.power_on(spi)?;
        self.display_refresh(spi)?;
        Ok(())
    }

    pub fn reset(&mut self, delay: &mut DELAY) -> Result<(), Box<dyn Error>> {
        delay.delay_ms(1);
        self.rst.set_high()?;
        delay.delay_ms(5);
        self.rst.set_low()?;
        delay.delay_ms(10);
        self.rst.set_high()?;
        delay.delay_ms(5);
        Ok(())
    }

    pub fn power_on(&mut self, spi: &mut SPI) -> Result<(), Box<dyn Error>> {
        self.send_data(spi, Command::PowerOn, &[0x0])?;
        self.wait_busy();
        Ok(())
    }

    pub fn power_off(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), Box<dyn Error>> {
        self.send_data(spi, Command::PowerOff, &[0x0])?;
        self.wait_busy();
        self.dc.set_low()?;
        delay.delay_ms(150);
        self.rst.set_low()?;
        Ok(())
    }

    fn send_data(
        &mut self,
        spi: &mut SPI,
        cmd: Command,
        data: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        self.dc.set_low()?;
        //TODO Implement single byte write or divide buffer into chunks. SPI hardware buffer might not be large enough for whole data.
        spi.write(&[cmd as u8])?;
        self.dc.set_high()?;
        spi.write(data)?;
        Ok(())
    }

    fn soft_reset(&mut self, spi: &mut SPI) -> Result<(), Box<dyn Error>> {
        self.send_data(spi, Command::Psr, REG_DATA_SOFT_RESET)?;
        self.wait_busy();
        Ok(())
    }

    fn display_refresh(&mut self, spi: &mut SPI) -> Result<(), Box<dyn Error>> {
        self.send_data(spi, Command::Refresh, &[0x0])?;
        self.wait_busy();
        Ok(())
    }

    fn wait_busy(&mut self) {
        while self.busy.is_low().unwrap() {}
    }
}
