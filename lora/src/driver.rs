#[cfg(feature = "defmt")]
use defmt::{debug, error};

use embedded_hal_async::spi::SpiDevice;
pub use sx127x_common::error::Sx127xError;
use sx127x_common::{Hz, Modem, CHIP_VERSION, DEFAULT_FREQUENCY_HZ, FSTEP};
use sx127x_common::bits::{get_bits, set_bits};
use sx127x_common::error::Sx127xError::InvalidVersion;
use sx127x_common::spi::Sx127xSpi;
use crate::calculate::{symbol_period, symbol_rate};
use crate::evaluate;
use crate::registers::*;
use crate::types::*;
use crate::validate;

#[cfg(feature = "half_duplex")]
const PAYLOAD_SIZE: usize = 256;
#[cfg(not(feature = "half_duplex"))]
const PAYLOAD_SIZE: usize = 128;

const HF_MIN_HZ: u32 = 779_000_000;

// TODO move this to types
pub struct Sx127xLoraConfig {
    pub bandwidth: Bandwidth,
    pub coding_rate: CodingRate,
    pub frequency: Hz,
    pub header_mode: HeaderMode,
    pub spreading_factor: SpreadingFactor,
    /// Whether or not to use the full automated (temperature-dependent) calibration.
    ///
    /// See: datasheet section 2.1.3.8
    pub use_auto_temp_calibration: bool,
    /// Whether or not to use the cyclic redundancy check (CRC) generation and verification on rx/tx payloads.
    pub use_crc: bool,
}
impl Sx127xLoraConfig {
    pub fn new(
        bandwidth: Bandwidth,
        coding_rate: CodingRate,
        frequency: Hz,
        header_mode: HeaderMode,
        spreading_factor: SpreadingFactor,
        use_auto_temp_calibration: bool,
        use_crc: bool,
    ) -> Result<Self, Sx127xError<()>> {
        if !validate::header_mode_sf(header_mode, spreading_factor) {
            #[cfg(feature = "defmt")]
            error!("SF6 requires implicit header mode");
            return Err(Sx127xError::InvalidInput);
        }
        Ok(Self { bandwidth, coding_rate, frequency, header_mode, spreading_factor, use_auto_temp_calibration, use_crc })
    }
}
impl Default for Sx127xLoraConfig {
    fn default() -> Self {
        Self {
            bandwidth: Bandwidth::default(),
            coding_rate: CodingRate::default(),
            frequency: DEFAULT_FREQUENCY_HZ,
            header_mode: HeaderMode::default(),
            spreading_factor: SpreadingFactor::default(),
            use_auto_temp_calibration: false,
            use_crc: false,
        }
    }
}

// rssi, snr, cr
pub struct RxPayload {
    pub coding_rate: CodingRate,
    pub payload: [u8; PAYLOAD_SIZE],
    pub rssi: u8,
    pub snr: i8,
}
impl RxPayload {
    pub(crate) fn new(coding_rate: CodingRate, payload: [u8; PAYLOAD_SIZE], rssi: u8, snr: i8) -> Self {
        Self { coding_rate, payload, rssi, snr }
    }
}

/// Sx127x driver with LoRa modem.
pub struct Sx127xLora<SPI> {
    pub spi: Sx127xSpi<SPI>
}
impl<SPI: SpiDevice> Sx127xLora<SPI> {
    pub async fn new(spi: SPI, config: Sx127xLoraConfig) -> Result<Sx127xLora<SPI>, Sx127xError<SPI::Error>> {
        let mut driver = Self { spi: Sx127xSpi::new(spi) };

        let version = driver.spi.read(VERSION).await?;
        if version != CHIP_VERSION {
            #[cfg(feature = "defmt")]
            error!("invalid chip version: {} != {}", version, CHIP_VERSION);
            return Err(InvalidVersion)
        }

        driver.set_modem(Modem::LoRa).await?;
        driver.config(config).await?;

        Ok(driver)
    }

    /// Triggers the IQ and RSSI calibration when set in Standby mode. Takes ~10ms.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn calibrate(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= IMAGE_CAL_IMAGE_CAL_START_MASK;
        self.write(IMAGE_CAL, image_cal).await
    }

    /// Clears all interrupts.
    pub async fn clear_all_interrupts(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(IRQ_FLAGS, 0xff).await
    }

    /// Configure band-specific registers 0x61-0x73 based upon the currently programmed frequency.
    ///
    /// See: datasheet section 4.3
    pub async fn config_band_specific_registers(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let frequency = self.frequency().await?;
        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, (frequency > HF_MIN_HZ) as u8, OP_MODE_LOW_FREQUENCY_MODE_ON_MASK, OP_MODE_LOW_FREQUENCY_MODE_ON_OFFSET);
        self.write(OP_MODE, byte).await
    }

    /// Sets the power amplifier (PA) to PA_HP on the PA_BOOST pin.
    ///
    /// See: datasheet section 3.4.2
    pub async fn config_tx(&mut self, config: TxConfig) -> Result<(), Sx127xError<SPI::Error>> {
        if config.use_rfo {
            self.write(PA_CONFIG, 0x70 | config.power).await?;
            self.write(PA_DAC, 0x04).await?;
        } else {
            self.write(PA_CONFIG, 0x80 | config.power).await?;
            self.write(PA_DAC, if config.power > 17 { 0x07 } else { 0x04 }).await?;
        }
        self.set_power_ramp(config.ramp).await
    }

    /// Clears an interrupt.
    pub async fn clear_interrupt<I: IRQ>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS).await?;
        self.write(IRQ_FLAGS, byte | <I as IRQ>::MASK).await
    }

    /// Gets the carrier frequency in Hz.
    ///
    /// See: datasheet section 4.1.4
    pub async fn frequency(&mut self) -> Result<u32, Sx127xError<SPI::Error>> {
        let msb = self.read(FRF_MSB).await? as u32;
        let mid = self.read(FRF_MID).await? as u32;
        let lsb = self.read(FRF_LSB).await? as u32;
        Ok((msb << 16) | (mid << 8) | lsb)
    }

    /// Gets the current hop channel.
    ///
    /// See: datasheet section 4.1.1.8
    pub async fn hop_channel(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        Ok(self.read(HOP_CHANNEL).await? & HOP_CHANNEL_FHSS_PRESENT_CHANNEL_MASK)
    }

    /// Gets the flag for interrupt `I`.
    pub async fn interrupt_flag<I: IRQ>(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        Ok(self.read(IRQ_FLAGS).await? & <I as IRQ>::MASK == 1)
    }

    /// Maps the DIO0 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn map_dio0<S: Dio0Signal>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.map_dio(DIO_MAPPING_1, <S as Dio0Signal>::VALUE, DIO_MAPPING_1_DIO0_MASK, DIO_MAPPING_1_DIO0_OFFSET).await
    }

    /// Maps the DIO1 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn map_dio1<S: Dio1Signal>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.map_dio(DIO_MAPPING_1, <S as Dio1Signal>::VALUE, DIO_MAPPING_1_DIO1_MASK, DIO_MAPPING_1_DIO1_OFFSET).await
    }

    /// Maps the DIO2 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn map_dio2<S: Dio2Signal>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.map_dio(DIO_MAPPING_1, <S as Dio2Signal>::VALUE, DIO_MAPPING_1_DIO2_MASK, DIO_MAPPING_1_DIO2_OFFSET).await
    }

    /// Maps the DIO3 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn map_dio3<S: Dio3Signal>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.map_dio(DIO_MAPPING_1, <S as Dio3Signal>::VALUE, DIO_MAPPING_1_DIO3_MASK, DIO_MAPPING_1_DIO3_OFFSET).await
    }

    /// Maps the DIO4 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn map_dio4<S: Dio4Signal>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.map_dio(DIO_MAPPING_2, <S as Dio4Signal>::VALUE, DIO_MAPPING_2_DIO4_MASK, DIO_MAPPING_2_DIO4_OFFSET).await
    }

    /// Maps the DIO5 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn map_dio5<S: Dio5Signal>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.map_dio(DIO_MAPPING_2, <S as Dio5Signal>::VALUE, DIO_MAPPING_2_DIO5_MASK, DIO_MAPPING_2_DIO5_OFFSET).await
    }

    /// Gets N bytes from the FIFO buffer, depending upon the `half_duplex` feature flag.
    ///
    /// See: datasheet figure 10
    pub async fn rx_payload(&mut self) -> Result<RxPayload, Sx127xError<SPI::Error>> {
        let reg_hop_channel = self.read(HOP_CHANNEL).await?;
        let crc_on_payload = get_bits(reg_hop_channel, HOP_CHANNEL_CRC_ON_PAYLOAD_MASK, HOP_CHANNEL_CRC_ON_PAYLOAD_OFFSET) == 1;

        let irq_flags_bits = self.read(IRQ_FLAGS).await? >> 4;
        let rx_packet_termination_ok = if crc_on_payload {
            #[cfg(feature = "defmt")]
            debug!("CRC on payload");
            irq_flags_bits & 0xf == 0
        } else {
            #[cfg(feature = "defmt")]
            debug!("CRC not on payload");
            irq_flags_bits & 0xc == 0 && irq_flags_bits & 0x1 == 0
        };
        if !rx_packet_termination_ok {
            #[cfg(feature = "defmt")]
            error!("RX packet termination failed");
            return Err(Sx127xError::PacketTermination)
        }

        let rx_fifo_addr = self.read(FIFO_RX_CURRENT_ADDR).await?;
        #[cfg(feature = "defmt")]
        debug!("rx_fifo_addr: {}", rx_fifo_addr);
        self.write(FIFO_ADDR_PTR, rx_fifo_addr).await?;

        let num_bytes = self.read(RX_NB_BYTES).await?;
        #[cfg(feature = "defmt")]
        debug!("num_bytes: {}", num_bytes);
        if num_bytes > PAYLOAD_SIZE as u8 {
            #[cfg(feature = "defmt")]
            error!("received {} bytes but buffer size is only {} bytes", num_bytes, PAYLOAD_SIZE);
            return Err(Sx127xError::InvalidPayloadLength)
        }

        let mut buffer = [0; PAYLOAD_SIZE];
        for i in 0..num_bytes {
            buffer[i as usize] = self.read(FIFO).await?;
        }

        let coding_rate = CodingRate::from(get_bits(self.read(MODEM_STAT).await?, MODEM_STAT_RX_CODING_RATE_MASK, MODEM_STAT_RX_CODING_RATE_OFFSET));
        let rssi = self.read(PKT_RSSI_VALUE).await?;
        let snr = self.read(PKT_SNR_VALUE).await? as i8 >> 2;

        Ok(RxPayload::new(coding_rate, buffer, rssi, snr))
    }

    /// Enables receive mode and searches for a preamble. If a `timeout` is specified, the device
    /// enter RXSINGLE mode, else RXCONTINUOUS mode.
    ///
    /// See: datasheet pages 40-42
    // TODO? p.40 "It is therefore necessary for the companion microcontroller to handle the address pointer to make sure the FIFO data buffer is never full"
    pub async fn rx(&mut self, timeout: Option<TimeoutSymbols>) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut mode = DeviceMode::RXCONTINUOUS;

        if let Some(timeout) = timeout {
            mode = DeviceMode::RXSINGLE;

            let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
            set_bits(&mut modem_config_2, (timeout.0 >> 8) as u8, MODEM_CONFIG_2_SYMB_TIMEOUT_MASK, MODEM_CONFIG_2_SYMB_TIMEOUT_OFFSET);
            self.write(MODEM_CONFIG_2, modem_config_2).await?;

            self.write(SYMB_TIMEOUT_LSB, (timeout.0 & 0xff) as u8).await?;
        }

        self.write(FIFO_ADDR_PTR, FIFO_RX_BASE_ADDR_VALUE).await?;
        self.set_device_mode(mode).await
    }

    /// Sets the temperature monitor operation flag. This will switch to the FSK/OOK modem,
    /// set/unset the temp monitor flag, then switch back to the LoRa modem before returning.
    ///
    /// See: datasheet section 2.1.3.8
    async fn set_auto_temp_calibration(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_modem(Modem::Fsk).await?;
        let image_cal = self.read(IMAGE_CAL).await?;
        self.write(IMAGE_CAL, image_cal | !on as u8).await?;
        self.set_modem(Modem::LoRa).await
    }

    /// Sets cyclic redundancy check (CRC) generation and verification on rx/tx payloads on/off.
    ///
    /// See: section 4.1.1.6
    pub async fn set_crc(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_MASK, MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_OFFSET);
        self.write(MODEM_CONFIG_2, byte).await
    }

    /// Sets the carrier frequency.
    ///
    /// See: datasheet section 4.1.4, datasheet tables 43-44
    pub async fn set_frequency(&mut self, hz: Hz) -> Result<(), Sx127xError<SPI::Error>> {
        let frf = sx127x_common::calculate::frf(hz, FSTEP);
        self.write(FRF_MSB, (frf >> 16) as u8).await?;
        self.write(FRF_MID, (frf >> 8) as u8).await?;
        self.write(FRF_LSB, frf as u8).await
    }

    /// Sets the symbol period between frequency hops. If `period` > 0 FHSS will be enabled.
    ///
    /// See: datasheet section 4.1.1.8
    pub async fn set_hop_period(&mut self, period: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(HOP_PERIOD, period).await
    }

    /// Starts the Channel Activity Detector (CAD).
    pub async fn start_cad(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::CAD).await
    }

    /// Transmits a `payload` of up to 255 bytes. Will automatically transition to STDBY when done.
    ///
    /// See: datasheet figure 9
    pub async fn tx(&mut self, payload: &[u8]) -> Result<(), Sx127xError<SPI::Error>> {
        let payload_len = payload.len();
        if payload_len > PAYLOAD_SIZE {
            #[cfg(feature = "defmt")]
            error!("payload len {} is greater than max {}", payload_len, PAYLOAD_SIZE);
            return Err(Sx127xError::InvalidPayloadLength);
        }

        self.write(FIFO_TX_BASE_ADDR, FIFO_TX_BASE_ADDR_VALUE).await?;

        self.set_device_mode(DeviceMode::STDBY).await?;
        self.write(FIFO_ADDR_PTR, FIFO_TX_BASE_ADDR_VALUE).await?;
        for &byte in payload.iter().take(PAYLOAD_SIZE) {
            self.write(FIFO, byte).await?;
        }
        self.write(PAYLOAD_LENGTH, payload_len as u8).await?;
        self.set_device_mode(DeviceMode::TX).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn bandwidth(&mut self) -> Result<Bandwidth, Sx127xError<SPI::Error>> {
        Ok(Bandwidth::from((self.read(MODEM_CONFIG_1).await? & MODEM_CONFIG_1_BW_MASK) >> MODEM_CONFIG_1_BW_OFFSET))
    }

    async fn config(&mut self, config: Sx127xLoraConfig) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_bandwidth(config.bandwidth).await?;
        self.set_coding_rate(config.coding_rate).await?;
        self.set_frequency(config.frequency).await?;
        self.set_header_mode(config.header_mode).await?;
        self.set_spreading_factor(config.spreading_factor).await?;
        self.set_auto_temp_calibration(config.use_auto_temp_calibration).await?;
        self.set_crc(config.use_crc).await?;

        if config.frequency != DEFAULT_FREQUENCY_HZ {
            self.calibrate().await?;
        }
        Ok(())
    }

    async fn header_mode(&mut self) -> Result<HeaderMode, Sx127xError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(HeaderMode::from(get_bits(byte, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_OFFSET)))
    }

    async fn map_dio(&mut self, register: u8, value: u8, mask: u8, offset: u8) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(register).await?;
        set_bits(&mut byte, value, mask, offset);
        self.write(register, byte).await
    }

    /// See: errata section 2.1
    async fn optimize_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        if bandwidth == Bandwidth::Bw500kHz {
            match self.frequency().await? {
                410_000_000..=525_000_000 => {
                    self.write(HIGH_BW_OPTIMIZE_1, 0x02).await?;
                    self.write(HIGH_BW_OPTIMIZE_2, 0x7f).await
                }
                862_000_000..=1_020_000_000 => {
                    self.write(HIGH_BW_OPTIMIZE_1, 0x02).await?;
                    self.write(HIGH_BW_OPTIMIZE_2, 0x64).await
                }
                _ => Ok(())
            }
        } else {
            self.write(HIGH_BW_OPTIMIZE_1, 0x03).await
        }
    }

    /// Enables/disables low data rate optimization.
    ///
    /// See: datasheet section 4.1.1.6
    async fn optimize_data_rate(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let on = self.should_optimize_low_data_rate().await?;

        let mut byte = self.read(MODEM_CONFIG_3).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_MASK, MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_OFFSET);
        self.write(MODEM_CONFIG_3, byte).await
    }

    async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(addr).await
    }

    /// Sets the bandwidth and then optimizes the sensitivity of the modem.
    ///
    /// See: datasheet section 4.1.1.4
    async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, bandwidth as u8, MODEM_CONFIG_1_BW_MASK, MODEM_CONFIG_1_BW_OFFSET);
        self.write(MODEM_CONFIG_1, byte).await?;

        self.optimize_bandwidth(bandwidth).await?;
        self.optimize_data_rate().await // TODO this reads bw again
    }

    /// Sets the cyclic error coding rate.
    ///
    /// See: datasheet section 4.1.1.3
    async fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, coding_rate as u8, MODEM_CONFIG_1_CODING_RATE_MASK, MODEM_CONFIG_1_CODING_RATE_OFFSET);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the device mode.
    ///
    /// See: datasheet table 16
    async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, OP_MODE_MODE_OFFSET);
        self.write(OP_MODE, byte).await
    }

    /// Sets the header mode to explicit or implicit.
    ///
    /// See: datasheet section 4.1.1.6
    async fn set_header_mode(&mut self, mode: HeaderMode) -> Result<(), Sx127xError<SPI::Error>> {
        let sf = self.spreading_factor().await?;
        if !validate::header_mode_sf(mode, sf) {
            #[cfg(feature = "defmt")]
            error!("SF6 requires implicit header mode");
            return Err(Sx127xError::InvalidInput);
        }

        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, mode as u8, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_OFFSET);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the rise/fall time of the power amplifier (PA).
    async fn set_power_ramp(&mut self, pa_ramp: PowerRamp) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(PA_RAMP).await?;
        self.write(PA_RAMP, byte | pa_ramp as u8).await
    }

    /// Sets the spreading factor. If SF6, implicit header mode must already be set.
    ///
    /// See: datasheet section 4.1.1.2
    async fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<(), Sx127xError<SPI::Error>> {
        if spreading_factor == SpreadingFactor::Sf6 {
            if self.header_mode().await? != HeaderMode::Implicit {
                #[cfg(feature = "defmt")]
                error!("SF6 requires implicit header mode");
                return Err(Sx127xError::SF6RequiresImplicitHeaderMode);
            }
        }

        let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut modem_config_2, spreading_factor as u8, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, MODEM_CONFIG_2_SPREADING_FACTOR_OFFSET);
        self.write(MODEM_CONFIG_2, modem_config_2).await?;

        let mut detect_optimize = self.read(DETECT_OPTIMIZE).await?;
        detect_optimize &= !DETECT_OPTIMIZE_DETECTION_OPTIMIZE_MASK;

        if spreading_factor == SpreadingFactor::Sf6 {
            detect_optimize |= DETECT_OPTIMIZE_DETECTION_OPTIMIZE_SF6;
            self.write(DETECTION_THRESHOLD, DETECTION_THRESHOLD_SF6).await?;
        } else {
            detect_optimize |= DETECT_OPTIMIZE_DETECTION_OPTIMIZE_SF7_TO_SF12;
            self.write(DETECTION_THRESHOLD, DETECTION_THRESHOLD_SF7_TO_SF12).await?;
        }
        self.write(DETECT_OPTIMIZE, detect_optimize).await?;
        self.optimize_data_rate().await // TODO this reads sf again
    }

    /// Determines if low data rate optimization is necessary.
    ///
    /// See: datasheet section 4.1.1.6
    async fn should_optimize_low_data_rate(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        Ok(
            evaluate::should_optimize_for_low_data_rate(
                symbol_period(
                    symbol_rate(
                        self.bandwidth().await?.hz(),
                        self.spreading_factor().await? as u32
                    )
                )
            )
        )
    }

    /// Gets the spreading factor.
    async fn spreading_factor(&mut self) -> Result<SpreadingFactor, Sx127xError<SPI::Error>> {
        Ok(SpreadingFactor::from(get_bits(self.read(MODEM_CONFIG_2).await?, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, MODEM_CONFIG_2_SPREADING_FACTOR_OFFSET)))
    }

    async fn set_modem(&mut self, modem: Modem) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, modem as u8, OP_MODE_LONG_RANGE_MODE_MASK, OP_MODE_LONG_RANGE_MODE_OFFSET);
        self.write(OP_MODE, byte).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }

    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(addr, data).await
    }
}