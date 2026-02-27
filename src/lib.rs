#![no_std]

mod registers;
mod types;

use embedded_hal_async::spi::SpiDevice;
use crate::registers::*;
use crate::types::{CyclicErrorCoding, DeviceMode, SpreadingFactor};

pub enum Sx127xError<SPI> {
    SPI(SPI),
}

/// LoRa driver.
pub struct Sx127x<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice>Sx127x<SPI> {

    // enable_shared_reg_access

    /// Sets the coding rate.
    // TODO "In implicit header mode should be set on receiver to determine expected coding rate. See 4.1.1.3"
    // pub async fn set_coding_rate(&mut self, rate: CyclicErrorCoding) -> Result<(), Sx127xError<SPI::Error>> {
    //     self.set_register_bits(REG_MODEM_CONFIG_1, REG_MODEM_CONFIG_1_CODING_RATE_MASK, rate as u8).await
    // }
    //
    // pub async fn set_header_mode(&mut self, implicit: bool) -> Result<(), Sx127xError<SPI::Error>> {
    //     self.set_register_bits(REG_MODEM_CONFIG_1, REG_MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, implicit as u8).await
    // }

    // set_low_frequency_mode(&mut self, on: bool)

    /// Sets the spreading factor.
    // pub async fn set_spreading_factor(&mut self, factor: SpreadingFactor) -> Result<(), Sx127xError<SPI::Error>> {
    //     self.set_register_bits(REG_MODEM_CONFIG_2, REG_MODEM_CONFIG_2_SPREADING_FACTOR_MASK, (factor as u8) << 4).await?;
    //     if factor == SpreadingFactor::Factor6 {
    //         self.set_header_mode(true).await?;
    //         self.set_register_bits(REG_DETECT_OPTIMIZE, REG_DETECT_OPTIMIZE_DETECT_OPTIMIZE_MASK, 0x5).await?;
    //         self.write(REG_DETECTION_THRESHOLD, 0x0c).await
    //     } else {
    //         // TODO set explicit header mode?
    //         self.set_register_bits(REG_DETECT_OPTIMIZE, REG_DETECT_OPTIMIZE_DETECT_OPTIMIZE_MASK, 0x3).await?;
    //         self.write(REG_DETECTION_THRESHOLD, 0x0a).await
    //     }
    // }

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

    // SPI -----------------------------------------------------------------------------------------

    /// Reads the byte from the register at `addr`.
    async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        let mut read = [0u8; 2];
        // other module: let write = [reg & 0x7f, 0]; 0x7f == 0111_1111 // TODO why?
        let write = [addr, 0u8];
        self.spi.transfer(&mut read, &write).await.map_err(Sx127xError::SPI)?;
        Ok(read[1])
    }

    /// Writes a raw byte `data` to register at `addr`.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        // other module: let buffer = [reg | 0x80, byte]; 0x80 == 1000_0000 // TODO why?
        let buf = [addr, data];
        self.spi.write(&buf).await.map_err(Sx127xError::SPI)
    }
}