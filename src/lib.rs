#![no_std]

mod registers;
mod types;

use embedded_hal_async::spi::SpiDevice;
use crate::registers::*;
use crate::types::{DeviceMode, RxStatus};

//const FXOSC_HHZ: u32 = 32;

pub enum Sx127xError<SPI> {
    SPI(SPI),
}

/// LoRa driver.
pub struct Sx127x<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice>Sx127x<SPI> {

    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Reads the byte from the register at `addr`.
    pub async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        let mut read = [0u8; 2];
        // other module: let write = [reg & 0x7f, 0]; 0x7f == 0111_1111 // TODO why?
        let write = [addr, 0u8];
        self.spi.transfer(&mut read, &write).await.map_err(Sx127xError::SPI)?;
        Ok(read[1])
    }

    /// Reads the RX modem status.
    pub async fn rx_status(&mut self) -> Result<RxStatus, Sx127xError<SPI::Error>> {
        let byte = RegModemStat::from_bits(self.read(RegModemStat::addr()).await?);
        Ok(RxStatus::from(byte))
    }

    /// Puts the device in sleep mode.
    pub async fn sleep(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::Sleep).await
    }

    /// Puts the device in standby mode.
    pub async fn standby(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::Stdby).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    /// Sets `device_mode` on RegOpMode.
    async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte: RegOpMode = RegOpMode::from_bits(self.read(RegOpMode::addr()).await?);
        byte.set_mode(device_mode);
        self.write(RegOpMode::addr(), byte.into_bits()).await
    }

    /// Writes the `data` raw byte to the register at `addr`.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        // other module: let buffer = [reg | 0x80, byte]; 0x80 == 1000_0000 // TODO why?
        let buf = [addr, data];
        self.spi.write(&buf).await.map_err(Sx127xError::SPI)
    }
}