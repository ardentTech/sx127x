#![no_std]

mod registers;
mod types;

pub use types::{Dio0, Dio1};
pub use types::Interrupt;

use embedded_hal_async::spi::SpiDevice;
use crate::registers::*;
use crate::types::*;
use crate::types::DeviceMode::{RxContinuous, RxSingle};

const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;
const FXOSC_HZ: u32 = 32_000_000;
const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);

#[derive(Debug)]
pub enum Sx127xError<SPI> {
    InvalidPayloadLength,
    InvalidState,
    InvalidSymbolTimeout,
    PacketTermination,
    SPI(SPI),
}

pub struct Sx127xConfig {
    pub bandwidth: Bandwidth,
    pub coding_rate: CyclicErrorCoding,
    pub frequency: u32, // Hz
    pub spreading_factor: SpreadingFactor,
}
impl Default for Sx127xConfig {
    fn default() -> Self {
        Self {
            bandwidth: Bandwidth::default(),
            coding_rate: CyclicErrorCoding::default(),
            frequency: DEFAULT_FREQUENCY_HZ,
            spreading_factor: SpreadingFactor::default(),
        }
    }
}

/// Sx127x driver with LoRa modem.
pub struct Sx127x<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice>Sx127x<SPI> {

    pub async fn new(spi: SPI, config: Sx127xConfig) -> Result<Sx127x<SPI>, Sx127xError<SPI::Error>> {
        let mut driver = Self { spi };

        if config.bandwidth != Bandwidth::default() {
            driver.set_bandwidth(config.bandwidth).await?;
        }
        if config.coding_rate != CyclicErrorCoding::default() {
            driver.set_coding_rate(config.coding_rate).await?;
        }
        if config.frequency != DEFAULT_FREQUENCY_HZ {
            driver.set_frequency(config.frequency).await?;
        }
        if config.spreading_factor != SpreadingFactor::default() {
            driver.set_spreading_factor(config.spreading_factor).await?;
        }

        driver.sleep().await?;
        let mut byte = RegOpMode::from_bits(driver.read(RegOpMode::addr()).await?);
        byte.set_long_range_mode(true);
        driver.write(RegOpMode::addr(), byte.into_bits()).await?;
        driver.standby().await?; // TODO leave in Sleep?
        Ok(driver)
    }

    /// Clears an interrupt if it was triggered.
    pub async fn clear_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegIrqFlags::from_bits(self.read(RegIrqFlags::addr()).await?);
        if byte.interrupt_triggered(interrupt) {
            byte.clear_interrupt(interrupt);
            self.write(RegIrqFlags::addr(), byte.into_bits()).await
        } else {
            Ok(())
        }
    }

    /// Enables the DIO0 pin.
    pub async fn enable_dio0(&mut self, dio: Dio0) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegDioMapping1::from_bits(self.read(RegDioMapping1::addr()).await?);
        byte.set_dio0(dio as u8);
        self.write(RegDioMapping1::addr(), byte.into_bits()).await
    }

    /// Enables the DIO1 pin.
    pub async fn enable_dio1(&mut self, dio: Dio1) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegDioMapping1::from_bits(self.read(RegDioMapping1::addr()).await?);
        byte.set_dio1(dio as u8);
        self.write(RegDioMapping1::addr(), byte.into_bits()).await
    }

    /// Enables the DIO2 pin.
    pub async fn enable_dio2(&mut self, dio: Dio2) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegDioMapping1::from_bits(self.read(RegDioMapping1::addr()).await?);
        byte.set_dio2(dio as u8);
        self.write(RegDioMapping1::addr(), byte.into_bits()).await
    }

    /// Enables the DIO3 pin.
    pub async fn enable_dio3(&mut self, dio: Dio3) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegDioMapping1::from_bits(self.read(RegDioMapping1::addr()).await?);
        byte.set_dio3(dio as u8);
        self.write(RegDioMapping1::addr(), byte.into_bits()).await
    }

    /// Enables the DIO4 pin.
    pub async fn enable_dio4(&mut self, dio: Dio4) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegDioMapping2::from_bits(self.read(RegDioMapping2::addr()).await?);
        byte.set_dio4(dio as u8);
        self.write(RegDioMapping2::addr(), byte.into_bits()).await
    }

    /// Enables the DIO5 pin.
    pub async fn enable_dio5(&mut self, dio: Dio4) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegDioMapping2::from_bits(self.read(RegDioMapping2::addr()).await?);
        byte.set_dio5(dio as u8);
        self.write(RegDioMapping2::addr(), byte.into_bits()).await
    }

    /// Checks whether an interrupt was triggered.
    pub async fn interrupt_triggered(&mut self, interrupt: Interrupt) -> Result<bool, Sx127xError<SPI::Error>> {
        let byte = RegIrqFlags::from_bits(self.read(RegIrqFlags::addr()).await?);
        Ok(byte.interrupt_triggered(interrupt))
    }

    /// Masks an interrupt.
    pub async fn mask_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegIrqFlagsMask::from_bits(self.read(RegIrqFlagsMask::addr()).await?);
        byte.mask(interrupt);
        self.write(RegIrqFlagsMask::addr(), byte.into_bits()).await
    }

    /// Reads the byte from the register at `addr`.
    pub async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        let mut read = [0; 2];
        // 1 wnr bit (0 for read) + 7 bit addr
        let write = [addr & 0x7f, 0];
        self.spi.transfer(&mut read, &write).await.map_err(Sx127xError::SPI)?;
        Ok(read[1])
    }

    /// Reads 255 bytes from the FIFO buffer.
    pub async fn read_rx_data(&mut self) -> Result<[u8; 255], Sx127xError<SPI::Error>> {
        let reg_hop_channel = RegHopChannel::from_bits(self.read(RegHopChannel::addr()).await?);
        let reg_irq_flags = RegIrqFlags::from_bits(self.read(RegIrqFlags::addr()).await?);
        if !reg_irq_flags.packet_rx_termination_ok(reg_hop_channel.crc_on_payload()) {
            return Err(Sx127xError::PacketTermination)
        }

        // read rx data
        let rx_fifo_addr = self.read(Reg::FifoRxCurrentAddr as u8).await?;
        self.write(Reg::FifoAddrPtr as u8, rx_fifo_addr).await?;
        let num_bytes = self.read(Reg::RxNbBytes as u8).await?;
        let mut buffer = [0; 255];
        for i in 0..num_bytes {
            let byte = self.read(Reg::Fifo as u8).await?;
            buffer[i as usize] = byte;
        }
        // TODO reset FifoAddrPtr?
        Ok(buffer)
    }

    /// Enables receive mode and searches for a preamble.
    ///
    /// If `timeout` is not None, enters `RxSingle` device mode. Otherwise, enters `RxContinuous`
    /// device mode.
    pub async fn receive(&mut self, timeout: Option<u16>) -> Result<(), Sx127xError<SPI::Error>> {
        let mut mode = RxContinuous;
        if let Some(timeout) = timeout {
            // TODO unit test  (make this a tuple struct and put validation on it? easier to test?)s
            // if a struct (or other) could have MIN, MAX helpers...
            if timeout < 4 || timeout > 1023 {
                return Err(Sx127xError::InvalidSymbolTimeout)
            }

            // TODO test this
            self.write(RegModemConfig2::addr(), (timeout >> 8) as u8).await?;
            self.write(RegSymbTimeoutLsb::addr(), (timeout & 0xff) as u8 ).await?;
            mode = RxSingle;
        }
        self.standby().await?;
        self.write(Reg::FifoAddrPtr as u8, Reg::FifoRxBaseAddr as u8).await?;
        self.set_device_mode(mode).await?;
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
    ///
    /// See: page 27
    pub async fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<(), Sx127xError<SPI::Error>> {
        let mut modem_config_2 = RegModemConfig2::from_bits(self.read(RegModemConfig2::addr()).await?);
        modem_config_2.set_spreading_factor(spreading_factor);
        self.write(RegModemConfig2::addr(), modem_config_2.into_bits()).await?;

        if spreading_factor == SpreadingFactor::Sf6 {
            self.set_header_mode(true).await?;
        }
        let mut detect_optimize = RegDetectOptimize::from_bits(self.read(RegDetectOptimize::addr()).await?);
        detect_optimize.update(spreading_factor);
        self.write(RegDetectOptimize::addr(), detect_optimize.into_bits()).await?;

        // TODO this feels a bit heavy-handed
        let mut detection_threshold = RegDetectionThreshold::from_bits(self.read(RegDetectionThreshold::addr()).await?);
        detection_threshold.update(spreading_factor);
        self.write(RegDetectionThreshold::addr(), detection_threshold.into_bits()).await?;

        Ok(())
    }

    /// Puts the device in sleep mode, which clears the FIFO buffer.
    pub async fn sleep(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::Sleep).await
    }

    /// Puts the device in standby mode.
    pub async fn standby(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::Stdby).await
    }

    /// Transmits a `payload` of up to 255 bytes.
    pub async fn transmit(&mut self, payload: &[u8]) -> Result<(), Sx127xError<SPI::Error>> {
        let payload_len = payload.len();
        if payload_len > 255 {
            return Err(Sx127xError::InvalidPayloadLength);
        }

        // The chip will automatically transition the state to Stdby when done.
        let device_mode = RegOpMode::from_bits(self.read(RegOpMode::addr()).await?).mode();
        // TODO is this sufficient? MUST is be Standby?
        if device_mode == DeviceMode::Tx {
            return Err(Sx127xError::InvalidState)
        }

        self.standby().await?;
        self.write(Reg::FifoAddrPtr as u8, Reg::FifoTxBaseAddr as u8).await?;
        for &byte in payload.iter().take(255) {
            self.write(Reg::Fifo as u8, byte).await?;
        }
        self.write(Reg::PayloadLength as u8, payload.len() as u8).await?;
        self.set_device_mode(DeviceMode::Tx).await
    }

    /// Unmasks an interrupt.
    pub async fn unmask_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = RegIrqFlagsMask::from_bits(self.read(RegIrqFlagsMask::addr()).await?);
        byte.unmask(interrupt);
        self.write(RegIrqFlagsMask::addr(), byte.into_bits()).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn get_reg_modem_config_1(&mut self) -> Result<RegModemConfig1, Sx127xError<SPI::Error>> {
        Ok(RegModemConfig1::from_bits(self.read(RegModemConfig1::addr()).await?))
    }

    // Sets `device_mode` on RegOpMode.
    async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte: RegOpMode = RegOpMode::from_bits(self.read(RegOpMode::addr()).await?);
        byte.set_mode(device_mode);
        self.write(RegOpMode::addr(), byte.into_bits()).await
    }

    // Writes the `data` raw byte to the register at `addr`.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        // 1 wnr bit (1 for write) + 7 bit addr
        let buf = [addr | 0x80, data];
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