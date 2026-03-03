#![no_std]

mod registers;
mod types;

use embedded_hal_async::spi::SpiDevice;
use crate::registers::*;
use crate::types::{DeviceMode, RxStatus};

const FXOSC_HZ: u32 = 32_000_000;
const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);

#[derive(Debug)]
pub enum Sx127xError<SPI> {
    DeviceBusy,
    InvalidPayloadLength,
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

    /// Sets the carrier frequency.
    ///
    /// Important: check regulations for your area (e.g. 902-928 MHz for the United States)
    ///
    /// Default is 434 MHz.
    pub async fn set_frequency(&mut self, hz: u32) -> Result<(), Sx127xError<SPI::Error>> {
        let frf = calculate_frf(hz);
        self.write(Reg::FrMsb as u8, (frf >> 16) as u8).await?;
        self.write(Reg::FrMid as u8, (frf >> 8) as u8).await?;
        self.write(Reg::FrLsb as u8, frf as u8).await
    }

    /// Puts the device in sleep mode.
    pub async fn sleep(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::Sleep).await
    }

    /// Puts the device in standby mode.
    pub async fn standby(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::Stdby).await
    }

    /// Transmits a `payload` of up to 255 bytes.
    /// See: [DS figure 9]
    pub async fn transmit(&mut self, payload: &[u8]) -> Result<(), Sx127xError<SPI::Error>> {
        let payload_len = payload.len();
        if payload_len > 255 {
            return Err(Sx127xError::InvalidPayloadLength);
        }

        // The chip will automatically transition the state to Stdby when done.
        let device_mode = self.get_device_mode().await?;
        if device_mode == DeviceMode::Tx {
            return Err(Sx127xError::DeviceBusy)
        }

        self.standby().await?;
        self.write(Reg::FifoAddrPtr as u8, Reg::FifoTxBaseAddr as u8).await?;
        for &byte in payload.iter().take(255) {
            self.write(Reg::Fifo as u8, byte).await?;
        }
        self.write(Reg::PayloadLength as u8, payload.len() as u8).await?;
        self.set_device_mode(DeviceMode::Tx).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn get_device_mode(&mut self) -> Result<DeviceMode, Sx127xError<SPI::Error>> {
        Ok(RegOpMode::from_bits(self.read(RegOpMode::addr()).await?).mode())
    }

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

fn calculate_frf(hz: u32) -> u32 {
    ((hz as f32) / FSTEP) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculate_frf_ok() {
        let frf = calculate_frf(434_000_000);
        assert_eq!(frf, 0x6c8000);
    }
}