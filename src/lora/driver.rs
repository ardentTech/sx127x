use embedded_hal_async::spi::SpiDevice;
use crate::common::*;
use crate::lora::registers::*;
use crate::lora::types::*;

const BUFFER_SIZE: usize = 255;
const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;
const MIN_RX_TIMEOUT: u8 = 4; // symbols
const MAX_RX_TIMEOUT: u16 = 1023; // symbols
// identifies silicon Version 1b, which applies to errata
const PRODUCTION_VERSION: u8 = 0x12;

#[derive(Debug)]
pub enum Sx127xLoraError<SPI> {
    InvalidPayloadLength,
    InvalidState,
    InvalidTimeout,
    PacketTermination,
    SPI(SPI),
}

pub struct Sx127xConfig {
    pub bandwidth: Bandwidth,
    pub coding_rate: CodingRate,
    pub frequency: u32,
    pub spreading_factor: SpreadingFactor,
}
impl Default for Sx127xConfig {
    fn default() -> Self {
        Self {
            bandwidth: Bandwidth::default(),
            coding_rate: CodingRate::default(),
            frequency: DEFAULT_FREQUENCY_HZ,
            spreading_factor: SpreadingFactor::default(),
        }
    }
}
impl Sx127xConfig {
    pub fn new(
        bandwidth: Bandwidth,
        coding_rate: CodingRate,
        frequency: u32,
        spreading_factor: SpreadingFactor
    ) -> Self {
        Self { bandwidth, coding_rate, frequency, spreading_factor }
    }
}

pub struct Sx127xLora<SPI> {
    spi: Sx127xSpi<SPI>
}

impl<SPI: SpiDevice> Sx127xLora<SPI> {

    /// Initializes an instance of the LoRa modem driver.
    pub async fn new(spi: SPI, config: Sx127xConfig) -> Result<Sx127xLora<SPI>, Sx127xLoraError<SPI::Error>> {
        let mut driver = Sx127xLora { spi: Sx127xSpi::new(spi) };

        driver.set_frequency(config.frequency).await?;
        driver.set_bandwidth(config.bandwidth).await?;
        driver.set_coding_rate(config.coding_rate).await?;
        driver.set_spreading_factor(config.spreading_factor).await?;
        // this will put the modem into long range mode
        driver.set_temp_monitor(false).await?;
        Ok(driver)
    }

    /// Gets the bandwidth.
    pub async fn bandwidth(&mut self) -> Result<Bandwidth, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(Bandwidth::from(get_bits(byte, MODEM_CONFIG_1_BW_MASK, 4)))
    }

    /// Triggers the IQ and RSSI calibration when set in Standby mode. Takes ~10ms.
    pub async fn calibrate(&mut self) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= 0x40;
        self.write(IMAGE_CAL, image_cal).await?;
        Ok(())
    }

    /// Clears an interrupt.
    pub async fn clear_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS).await?;
        self.write(IRQ_FLAGS, byte | interrupt as u8).await
    }

    /// Gets the cyclic error coding rate.
    async fn coding_rate(&mut self) -> Result<CodingRate, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(CodingRate::from(get_bits(byte, MODEM_CONFIG_1_CODING_RATE_MASK, 1)))
    }

    /// Enables/disables CRC generation and check on packet payload.
    ///
    /// In implicit header mode, if CRC generation is needed it must be set on both TX and RX.
    ///
    /// In explicit header mode, if CRC generation is needed it must be set on the TX side only. The
    /// CRC will be recovered from the packet header on the RX side.
    pub async fn crc_generation(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_MASK, 2);
        self.write(MODEM_CONFIG_2, byte).await
    }

    /// Calculates the current data rate in bits/s.
    pub async fn data_rate(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let coding_rate: f32 = self.coding_rate().await?.into();
        let symbol_rate = self.symbol_rate().await? as f32;
        let spreading_factor = (self.spreading_factor().await? as u8) as f32;
        Ok(calculate_data_rate(symbol_rate, spreading_factor, coding_rate))
    }

    /// Gets the device mode.
    pub async fn device_mode(&mut self) -> Result<DeviceMode, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(OP_MODE).await?;
        Ok(DeviceMode::from(get_bits(byte, OP_MODE_MODE_MASK, 0)))
    }

    /// Gets the carrier frequency.
    pub async fn frequency(&mut self) -> Result<u32, Sx127xLoraError<SPI::Error>> {
        let msb = self.read(FRF_MSB).await?;
        let mid = self.read(FRF_MID).await?;
        let lsb = self.read(FRF_LSB).await?;
        Ok(((msb as u32) << 16) | (mid as u32) << 8 | lsb as u32)
    }

    /// Gets the header mode.
    pub async fn header_mode(&mut self) -> Result<HeaderMode, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(HeaderMode::from(get_bits(byte, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, 0)))
    }

    /// Masks an interrupt.
    pub async fn mask_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte | interrupt as u8).await
    }

    /// Gets the current modem status.
    pub async fn modem_status(&mut self) -> Result<ModemStatus, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_STAT).await?;
        Ok(ModemStatus::from(byte))
    }

    /// Gets the packet preamble length.
    pub async fn preamble_length(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let msb = self.read(PREAMBLE_MSB).await?;
        let lsb = self.read(PREAMBLE_LSB).await?;
        Ok(((msb as u16) << 8) | lsb as u16)
    }

    /// Gets the byte from register `addr`.
    pub async fn read(&mut self, addr: u8) -> Result<u8, Sx127xLoraError<SPI::Error>> {
        self.spi.read(addr).await.map_err(Sx127xLoraError::SPI)
    }

    /// Reads 255 bytes from the FIFO buffer.
    // TODO should this return an object with Rx metadata in addition to data?
    pub async fn read_rx_data(&mut self) -> Result<[u8; BUFFER_SIZE], Sx127xLoraError<SPI::Error>> {
        let reg_hop_channel = self.read(HOP_CHANNEL).await?;
        if !self.rx_packet_termination_ok((reg_hop_channel & HOP_CHANNEL_CRC_ON_PAYLOAD_MASK) != 0).await? {
            return Err(Sx127xLoraError::PacketTermination)
        }

        let rx_fifo_addr = self.read(FIFO_RX_CURRENT_ADDR).await?;
        self.write(FIFO_ADDR_PTR, rx_fifo_addr).await?;
        let num_bytes = self.read(RX_NB_BYTES).await?;
        let mut buffer = [0; 255];
        for i in 0..num_bytes {
            let byte = self.read(FIFO).await?;
            buffer[i as usize] = byte;
        }
        Ok(buffer)
    }

    /// Enables receive mode and searches for a preamble with an optional timeout (in symbols).
    ///
    /// If `timeout` is not None, enters `RxSingle` device mode. Otherwise, enters `RxContinuous`
    /// device mode.
    pub async fn receive(&mut self, timeout: Option<u16>) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut mode = DeviceMode::RXCONTINUOUS;
        if let Some(timeout) = timeout {
            if timeout < (MIN_RX_TIMEOUT as u16) || timeout > MAX_RX_TIMEOUT {
                return Err(Sx127xLoraError::InvalidTimeout)
            }

            self.write(MODEM_CONFIG_2, (timeout >> 8) as u8).await?;
            self.write(SYMB_TIMEOUT_LSB, (timeout & 0xff) as u8 ).await?;
            mode = DeviceMode::RXSINGLE;
        }
        self.set_device_mode(DeviceMode::STDBY).await?;
        self.write(FIFO_ADDR_PTR, FIFO_RX_BASE_ADDR).await?;
        self.set_device_mode(mode).await
    }

    /// Sets the modem bandwidth.
    pub async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, bandwidth as u8, MODEM_CONFIG_1_BW_MASK, 4);

        if bandwidth == Bandwidth::Bw500kHz {
            self.optimize_500khz_bandwidth().await?;
        }

        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the coding rate.
    pub async fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, coding_rate as u8, MODEM_CONFIG_1_CODING_RATE_MASK, 1);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the DIO0 pin signal source.
    pub async fn set_dio0(&mut self, signal: Dio0Signal) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_dio_mapping1(signal as u8, DIO_MAPPING_1_DIO0_MASK, DIO_MAPPING_1_DIO0_SHIFT).await
    }

    /// Sets the DIO1 pin signal source.
    pub async fn set_dio1(&mut self, signal: Dio1Signal) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_dio_mapping1(signal as u8, DIO_MAPPING_1_DIO1_MASK, DIO_MAPPING_1_DIO1_SHIFT).await
    }

    /// Sets the device mode.
    pub async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, 0);
        self.write(OP_MODE, byte).await
    }

    /// Sets the carrier frequency. It's critical that you check regulations for your area (e.g.
    /// 902-928 MHz for the United States)
    pub async fn set_frequency(&mut self, hz: u32) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let frf = calculate_frf(hz);
        self.write(FRF_MSB, (frf >> 16) as u8).await?;
        self.write(FRF_MID, (frf >> 8) as u8).await?;
        self.write(FRF_LSB, frf as u8).await?;

        self.calibrate().await
    }

    /// Sets the header mode to explicit or implicit.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn set_header_mode(&mut self, mode: HeaderMode) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, mode as u8, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, 0);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the packet preamble length.
    pub async fn set_preamble_length(&mut self, preamble_length: u16) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.write(PREAMBLE_MSB, (preamble_length >> 8) as u8).await?;
        self.write(PREAMBLE_LSB, preamble_length as u8).await
    }

    /// Sets the spreading factor.
    ///
    /// See: datasheet page 27
    pub async fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut modem_config_2, spreading_factor as u8, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, 4);
        self.write(MODEM_CONFIG_2, modem_config_2).await?;

        let mut detect_optimize = self.read(DETECT_OPTIMIZE).await?;
        detect_optimize &= !DETECT_OPTIMIZE_DETECTION_OPTIMIZE_MASK;

        if spreading_factor == SpreadingFactor::Sf6 {
            self.set_header_mode(HeaderMode::Implicit).await?;
            detect_optimize |= 0x5;
            self.write(DETECTION_THRESHOLD, 0x0c).await?;
        } else {
            detect_optimize |= 0x3;
            self.write(DETECTION_THRESHOLD, 0x0a).await?;
        }
        self.write(DETECT_OPTIMIZE, detect_optimize).await?;

        Ok(())
    }

    /// Disables the temperature monitor operation.
    ///
    /// see: datasheet section 2.1.3.8: "It is recommended to disable the fully automated
    /// (temperature-dependent) calibration, to better control when it is triggered (and avoid
    /// unexpected packet losses)"
    // TODO handle other temp params?
    pub async fn set_temp_monitor(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // enter FSK/OOK mode
        self.set_long_range_mode(false).await?;

        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= !(on as u8);
        self.write(IMAGE_CAL, image_cal).await?;

        self.set_long_range_mode(true).await
    }

    /// Gets the spreading factor.
    pub async fn spreading_factor(&mut self) -> Result<SpreadingFactor, Sx127xLoraError<SPI::Error>> {
        let modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        Ok(SpreadingFactor::from(get_bits(modem_config_2, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, 4)))
    }

    /// Calculates the current symbol rate in chips/s.
    pub async fn symbol_rate(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let modem_config_1 = self.read(MODEM_CONFIG_1).await?;
        let bandwidth = Bandwidth::from((modem_config_1 & MODEM_CONFIG_1_BW_MASK) >> 4).hz();
        let spreading_factor = self.spreading_factor().await?;

        Ok(calculate_symbol_rate(bandwidth, spreading_factor as u32) as u16)
    }

    /// Transmits a `payload` of up to 255 bytes.
    pub async fn transmit(&mut self, payload: &[u8]) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let payload_len = payload.len();
        if payload_len > BUFFER_SIZE {
            return Err(Sx127xLoraError::InvalidPayloadLength);
        }

        let op_mode = self.read(OP_MODE).await?;
        if op_mode & OP_MODE_MODE_MASK == DeviceMode::TX as u8 {
            return Err(Sx127xLoraError::InvalidState)
        }

        self.set_device_mode(DeviceMode::STDBY).await?;
        self.write(FIFO_ADDR_PTR, FIFO_TX_BASE_ADDR).await?;
        for &byte in payload.iter().take(255) {
            self.write(FIFO, byte).await?;
        }
        self.write(PAYLOAD_LENGTH, payload.len() as u8).await?;
        self.set_device_mode(DeviceMode::TX).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    // see: errata section 2.1
    async fn optimize_500khz_bandwidth(&mut self) -> Result<(), Sx127xLoraError<SPI::Error>> {
        if self.read(VERSION).await? != PRODUCTION_VERSION { return Ok(()) } // noop for engineering samples

        match self.frequency().await? {
            410_000_000..=525_000_000 => {
                self.write(HIGH_BW_OPTIMIZE_1, 0x02).await?;
                self.write(HIGH_BW_OPTIMIZE_2, 0x7f).await?;
            }
            862_000_000..=1_020_000_000 => {
                self.write(HIGH_BW_OPTIMIZE_1, 0x02).await?;
                self.write(HIGH_BW_OPTIMIZE_2, 0x64).await?;
            }
            _ => {
                self.write(HIGH_BW_OPTIMIZE_1, 0x03).await?;
            }
        }
        Ok(())
    }

    // Determines if a RX packet terminated successfully.
    async fn rx_packet_termination_ok(&mut self, crc_on_payload: bool) -> Result<bool, Sx127xLoraError<SPI::Error>> {
        let bits = self.read(IRQ_FLAGS).await? >> 4;
        Ok(if crc_on_payload {
            bits & 0xf == 0
        } else {
            bits & 0xc == 0 && bits & 0x1 == 0
        })
    }

    async fn set_dio_mapping1(&mut self, value: u8, mask: u8, left_shift: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(DIO_MAPPING_1).await?;
        set_bits(&mut byte, value, left_shift, mask);
        self.write(DIO_MAPPING_1, byte).await
    }

    // Selects the LoRa modem when `on` == true, and the FSK/OOK modem when `on` == false.
    async fn set_long_range_mode(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // also clears the FIFO buffer
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut op_mode = self.read(OP_MODE).await?;
        set_bits(&mut op_mode, OP_MODE_LONG_RANGE_MODE_MASK, if on { 0x80 } else { 0x0 }, 0);
        self.write(OP_MODE, op_mode).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }

    // Writes `data` to register `addr` over SPI.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.spi.write(addr, data).await.map_err(Sx127xLoraError::SPI)
    }
}