#![no_std]

mod registers;
mod types;

use embedded_hal_async::spi::SpiDevice;
use crate::registers::*;
use crate::types::{Bandwidth, CyclicErrorCoding, DeviceMode, Interrupt, RxStatus, SpreadingFactor};

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

    /// Clears an interrupt.
    pub async fn clear_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegIrqFlags::from_bits(self.read(RegIrqFlags::addr()).await?);
        // TODO check triggered before proceeding
        byte.clear_interrupt(interrupt);
        self.write(RegIrqFlags::addr(), byte.into_bits()).await
    }

    /// Checks whether an interrupt was triggered.
    pub async fn interrupt_triggered(&mut self, interrupt: Interrupt) -> Result<bool, Sx127xError<SPI::Error>> {
        let byte = RegIrqFlags::from_bits(self.read(RegIrqFlags::addr()).await?);
        Ok(byte.interrupt_triggered(interrupt))
    }

    // TODO mask_interrupt()

    /// Reads the byte from the register at `addr`.
    pub async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        let mut read = [0u8; 2];
        self.spi.transfer(&mut read, &[addr, 0u8]).await.map_err(Sx127xError::SPI)?;
        Ok(read[1])
    }

    // TODO receive_continuous

    // TODO timeout arg?
    pub async fn receive_single_blocking(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.standby().await?;
        self.write(Reg::FifoAddrPtr as u8, Reg::FifoRxBaseAddr as u8).await?;
        self.set_device_mode(DeviceMode::RxSingle).await?;
        // if explicit mode, ValidHeader interrupt will fire on reception of valid preamble
        // wait for IRQ (RxTimeout, RxDone)
        // chip will switch to standby
        // check PayloadCrcError for packet payload integrity
        // read rx data
        Ok(())
    }

    /// Reads the RX modem status.
    pub async fn rx_status(&mut self) -> Result<RxStatus, Sx127xError<SPI::Error>> {
        let byte = RegModemStat::from_bits(self.read(RegModemStat::addr()).await?);
        Ok(RxStatus::from(byte))
    }

    /// Sets the signal bandwidth.
    pub async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.get_reg_modem_config_1().await?;
        byte.set_bandwidth(bandwidth);
        self.write(RegModemConfig1::addr(), byte.into_bits()).await
    }

    /// Sets the cyclic error coding rate.
    async fn set_coding_rate(&mut self, coding_rate: CyclicErrorCoding) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.get_reg_modem_config_1().await?;
        byte.set_coding_rate(coding_rate);
        self.write(RegModemConfig1::addr(), byte.into_bits()).await
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

    /// Sets the header mode to implicit or explicit.
    pub async fn set_header_mode(&mut self, implicit: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.get_reg_modem_config_1().await?;
        byte.set_implicit_header_mode_on(implicit);
        self.write(RegModemConfig1::addr(), byte.into_bits()).await
    }

    /// Sets the spreading factor.
    pub async fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegModemConfig2::from_bits(self.read(RegModemConfig2::addr()).await?);
        byte.set_spreading_factor(spreading_factor);
        self.write(RegModemConfig2::addr(), byte.into_bits()).await
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
        let device_mode = RegOpMode::from_bits(self.read(RegOpMode::addr()).await?).mode();
        if device_mode == DeviceMode::Tx {
            return Err(Sx127xError::DeviceBusy) // TODO maybe Sx127xError::AlreadyBusy?
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

    async fn get_reg_modem_config_1(&mut self) -> Result<RegModemConfig1, Sx127xError<SPI::Error>> {
        Ok(RegModemConfig1::from_bits(self.read(RegModemConfig1::addr()).await?))
    }

    /// Sets `device_mode` on RegOpMode.
    async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte: RegOpMode = RegOpMode::from_bits(self.read(RegOpMode::addr()).await?);
        byte.set_mode(device_mode);
        self.write(RegOpMode::addr(), byte.into_bits()).await
    }

    /// Writes the `data` raw byte to the register at `addr`.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
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