use embedded_hal_async::spi::SpiDevice;
use crate::common::{calculate_data_rate, calculate_frf, calculate_symbol_rate, Sx127xSpi};
use crate::lora::registers::*;
use crate::lora::types::{Bandwidth, CodingRate, DeviceMode, Dio0Signal, Dio1Signal, Interrupt, SpreadingFactor};

const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;
const BUFFER_SIZE: usize = 255;

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

        if config.frequency != DEFAULT_FREQUENCY_HZ {
            driver.set_frequency(config.frequency).await?;
            driver.calibrate().await?;
        }

        driver.set_bandwidth(config.bandwidth).await?;
        driver.set_coding_rate(config.coding_rate).await?;
        driver.set_spreading_factor(config.spreading_factor).await?;

        driver.disable_temp_monitor().await?;
        driver.set_long_range_mode(true).await?;
        Ok(driver)
    }

    /// Clears an interrupt.
    pub async fn clear_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS).await?;
        self.write(IRQ_FLAGS, byte | interrupt as u8).await
    }

    /// Calculates the current data rate.
    pub async fn data_rate(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let coding_rate: f32 = self.coding_rate().await?.into();
        let symbol_rate = self.symbol_rate().await? as f32;
        let spreading_factor = (self.spreading_factor().await? as u8) as f32;
        Ok(calculate_data_rate(symbol_rate, spreading_factor, coding_rate))
    }

    /// Reads 255 bytes from the FIFO buffer.
    pub async fn read_rx_data(&mut self) -> Result<[u8; BUFFER_SIZE], Sx127xLoraError<SPI::Error>> {
        let reg_hop_channel = self.read(HOP_CHANNEL).await?;
        if !self.rx_packet_termination_ok((reg_hop_channel & HOP_CHANNEL_CRC_ON_PAYLOAD_MASK) != 0).await? {
            return Err(Sx127xLoraError::PacketTermination)
        }

        let rx_fifo_addr = self.read(FIFO_RX_CURRENT_ADDR).await?;
        self.write(FIFO_ADDR_PTR as u8, rx_fifo_addr).await?;
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
        //     // TODO unit test  (make this a tuple struct and put validation on it? easier to test?)s
        //     // if a struct (or other) could have MIN, MAX helpers...
            if timeout < 4 || timeout > 1023 {
                return Err(Sx127xLoraError::InvalidTimeout)
            }

            // TODO test this
            self.write(MODEM_CONFIG_2, (timeout >> 8) as u8).await?;
            self.write(SYMB_TIMEOUT_LSB, (timeout & 0xff) as u8 ).await?;
            mode = DeviceMode::RXSINGLE;
        }
        self.set_device_mode(DeviceMode::STDBY).await?;
        self.write(FIFO_ADDR_PTR, FIFO_RX_BASE_ADDR).await?;
        self.set_device_mode(mode).await
    }

    /// Sets the bandwidth.
    pub async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        byte &= !MODEM_CONFIG_1_BW_MASK;
        byte |= ((bandwidth as u8) << 4) & MODEM_CONFIG_1_BW_MASK;
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the coding rate.
    pub async fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        byte &= !MODEM_CONFIG_1_CODING_RATE_MASK;
        byte |= ((coding_rate as u8) << 1) & MODEM_CONFIG_1_CODING_RATE_MASK;
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
        byte &= !OP_MODE_MODE_MASK;
        byte |= (device_mode as u8) & OP_MODE_MODE_MASK;
        self.write(OP_MODE, byte).await
    }

    /// Sets the carrier frequency.
    ///
    /// Important: check regulations for your area (e.g. 902-928 MHz for the United States)
    pub async fn set_frequency(&mut self, hz: u32) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let frf = calculate_frf(hz);
        self.write(FRF_MSB, (frf >> 16) as u8).await?;
        self.write(FRF_MID, (frf >> 8) as u8).await?;
        self.write(FRF_LSB, frf as u8).await
    }

    /// Sets the spreading factor.
    ///
    /// See: page 27
    pub async fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        modem_config_2 &= !MODEM_CONFIG_2_SPREADING_FACTOR_MASK;
        modem_config_2 |= ((spreading_factor as u8) << 4) & MODEM_CONFIG_2_SPREADING_FACTOR_MASK;
        self.write(MODEM_CONFIG_2, modem_config_2).await?;

        let mut detect_optimize = self.read(DETECT_OPTIMIZE).await?;
        detect_optimize &= !DETECT_OPTIMIZE_DETECTION_OPTIMIZE_MASK;

        if spreading_factor == SpreadingFactor::Sf6 {
            self.set_header_mode(true).await?;
            detect_optimize |= 0x5;
            self.write(DETECTION_THRESHOLD, 0x0c).await?;
        } else {
            detect_optimize |= 0x3;
            self.write(DETECTION_THRESHOLD, 0x0a).await?;
        }
        self.write(DETECT_OPTIMIZE, detect_optimize).await?;

        Ok(())
    }

    /// Calculates the current symbol rate.
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

    // Disables temp monitor operation.
    //
    // see section 2.1.3.8: "It is recommended to disable the fully automated
    // (temperature-dependent) calibration, to better control when it is triggered (and avoid
    // unexpected packet losses)"
    async fn disable_temp_monitor(&mut self) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // enter FSK/OOK mode
        self.set_long_range_mode(false).await?;

        // turn temp monitor off
        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= 0x1;
        self.write(IMAGE_CAL, image_cal).await?;

        // TODO is this necessary?
        self.set_device_mode(DeviceMode::STDBY).await
    }

    // Triggers the IQ and RSSI calibration when set in Standby mode. Takes ~10ms.
    async fn calibrate(&mut self) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= 0x40;
        self.write(IMAGE_CAL, image_cal).await?;
        Ok(())
    }

    async fn coding_rate(&mut self) -> Result<CodingRate, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(CodingRate::from((byte & MODEM_CONFIG_1_CODING_RATE_MASK) >> 1))
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

    // Reads from register `addr` over SPI.
    async fn read(&mut self, addr: u8) -> Result<u8, Sx127xLoraError<SPI::Error>> {
        self.spi.read(addr).await.map_err(Sx127xLoraError::SPI)
    }

    async fn set_dio_mapping1(&mut self, value: u8, mask: u8, left_shift: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(DIO_MAPPING_1).await?;
        byte &= !mask;
        byte |= (value << left_shift) & mask;
        self.write(DIO_MAPPING_1, byte).await
    }

    async fn set_header_mode(&mut self, implicit: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        byte &= !MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK;
        byte |= (implicit as u8) & MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK;
        self.write(MODEM_CONFIG_1, byte).await
    }

    // Selects the LoRa modem when `on` == true, and the FSK/OOK modem when `on` == false.
    async fn set_long_range_mode(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // also clears the FIFO buffer
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut op_mode = self.read(OP_MODE).await?;
        op_mode &= !OP_MODE_LONG_RANGE_MODE_MASK;
        op_mode |= (if on { 0x80 } else { 0x0 }) & OP_MODE_LONG_RANGE_MODE_MASK;
        self.write(OP_MODE, op_mode).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }

    async fn spreading_factor(&mut self) -> Result<SpreadingFactor, Sx127xLoraError<SPI::Error>> {
        let modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        Ok(SpreadingFactor::from((modem_config_2 & MODEM_CONFIG_2_SPREADING_FACTOR_MASK) >> 4))
    }

    // Writes `data` to register `addr` over SPI.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.spi.write(addr, data).await.map_err(Sx127xLoraError::SPI)
    }
}