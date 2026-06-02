#[cfg(feature = "defmt")]
use defmt::{debug, error};

use embedded_hal_async::spi::SpiDevice;
pub use sx127x_common::error::Sx127xError;
use sx127x_common::{Hz, Modem, CHIP_VERSION, DEFAULT_FREQUENCY_HZ, FSTEP};
use sx127x_common::bits::{get_bits, set_bits};
use sx127x_common::error::Sx127xError::{InvalidState, InvalidVersion};
use sx127x_common::spi::Sx127xSpi;
use crate::{calculate, check};
use crate::constants::{HF_MIN_HZ, PAYLOAD_SIZE};
use crate::registers::*;
use crate::types::*;
use crate::validate;

// -------------------------------------------------------------------------------------------------
/// Sx127x driver with LoRa modem.
pub struct Sx127xLora<SPI> {
    pub spi: Sx127xSpi<SPI>
}
impl<SPI: SpiDevice> Sx127xLora<SPI> {
    /// Initializes a new instance of the loRa driver.
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

    /// Configures RX settings.
    pub async fn config_rx(&mut self, config: RxConfig) -> Result<(), Sx127xError<SPI::Error>> {
        //self.set_invert_iq(config.invert_iq, INVERT_IQ_RX_MASK, INVERT_IQ_RX_OFFSET).await?;
        self.set_optimize_rx_response(config.optimize_response).await?;
        self.set_preamble_length(config.preamble_length).await
    }

    /// Configures TX settings.
    pub async fn config_tx(&mut self, config: TxConfig) -> Result<(), Sx127xError<SPI::Error>> {
        //self.set_invert_iq(config.invert_iq, INVERT_IQ_TX_MASK, INVERT_IQ_TX_OFFSET).await?;
        if config.use_rfo {
            self.write(PA_CONFIG, 0x70 | config.power).await?;
            self.write(PA_DAC, 0x04).await?;
        } else {
            self.write(PA_CONFIG, 0x80 | config.power).await?;
            self.write(PA_DAC, if config.power > 17 { 0x07 } else { 0x04 }).await?;
        }
        self.set_power_ramp(config.ramp).await?;
        self.set_preamble_length(config.preamble_length).await?;
        self.set_ocp(config.ocp).await
    }

    /// Clears an interrupt.
    pub async fn clear_interrupt<I: IRQ>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS).await?;
        self.write(IRQ_FLAGS, byte | <I as IRQ>::MASK).await
    }

    /// Calculates the data rate in bits/s.
    pub async fn data_rate(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let coding_rate: f32 = self.coding_rate().await?.into();
        let symbol_rate = self.symbol_rate().await? as f32;
        let spreading_factor = (self.spreading_factor().await? as u8) as f32;
        Ok(calculate::data_rate(symbol_rate, spreading_factor, coding_rate))
    }

    /// Calculates the frequency error indicatino (FEI) in Hz and ppm.
    pub async fn fei(&mut self) -> Result<FEI, Sx127xError<SPI::Error>> {
        let msb = self.read(FEI_MSB).await?;
        let mid = self.read(FEI_MID).await?;
        let lsb = self.read(FEI_LSB).await?;
        let fei = (((msb as u32) << 16) | ((mid as u32) << 8) | (lsb as u32)) as i32;

        let bandwidth = self.bandwidth().await?;
        let frequency = self.frequency().await?;

        Ok(FEI::new(bandwidth, fei, frequency))
    }

    /// Gets the carrier frequency in Hz.
    ///
    /// See: datasheet section 4.1.4
    pub async fn frequency(&mut self) -> Result<Hz, Sx127xError<SPI::Error>> {
        let msb = self.read(FRF_MSB).await? as u32;
        let mid = self.read(FRF_MID).await? as u32;
        let lsb = self.read(FRF_LSB).await? as u32;
        let frf = ((msb << 16) | (mid << 8) | lsb) as f32;
        Ok((frf * FSTEP) as Hz)
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

    /// Masks an interrupt
    ///
    /// See: datasheet section 4.1.2.4
    pub async fn mask_interrupt<I: IRQ>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte | <I as IRQ>::MASK).await
    }

    /// Gets the received signal strength indicator (RSSI) in dBm of the last packet received.
    ///
    /// See: datasheet section 3.5.5
    pub async fn last_packet_rssi(&mut self) -> Result<i16, Sx127xError<SPI::Error>> {
        // TODO p87 note3
        Ok(calculate::last_packet_rssi_dbm(
            self.frequency().await?,
            self.read(PKT_RSSI_VALUE).await? as i16,
            self.last_packet_snr().await?,
            self.read(RSSI_VALUE).await? as i16
        ))
    }

    pub async fn last_packet_snr(&mut self) -> Result<i16, Sx127xError<SPI::Error>> {
        Ok((self.read(PKT_SNR_VALUE).await? >> 2) as i16)
    }

    /// Gets the received signal strength indicator (RSSI) in dBm.
    ///
    /// See: datasheet section 3.5.5
    pub async fn rssi(&mut self) -> Result<i16, Sx127xError<SPI::Error>> {
        Ok(calculate::rssi_dbm(self.frequency().await?, self.read(RSSI_VALUE).await? as i16))
    }

    /// Gets the received signal strength indicator (RSSI) wideband measurement.
    pub async fn rssi_wideband(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.read(RSSI_WIDEBAND).await
    }

    /// Gets N bytes from the FIFO buffer, depending upon the `half_duplex` feature flag.
    ///
    /// See: datasheet figure 10
    pub async fn rx_packet(&mut self) -> Result<RxPacket, Sx127xError<SPI::Error>> {
        let reg_hop_channel = self.read(HOP_CHANNEL).await?;
        let crc_on_payload = get_bits(reg_hop_channel, HOP_CHANNEL_CRC_ON_PAYLOAD_MASK, HOP_CHANNEL_CRC_ON_PAYLOAD_OFFSET) == 1;

        let irq_flags_bits = self.read(IRQ_FLAGS).await? >> 4;
        let rx_packet_termination_ok = if crc_on_payload {
            irq_flags_bits & 0xf == 0
        } else {
            irq_flags_bits & 0xc == 0 && irq_flags_bits & 0x1 == 0
        };
        if !rx_packet_termination_ok {
            #[cfg(feature = "defmt")]
            error!("RX packet termination failed");
            return Err(Sx127xError::PacketTermination)
        }

        let rx_fifo_addr = self.read(FIFO_RX_CURRENT_ADDR).await?;
        self.write(FIFO_ADDR_PTR, rx_fifo_addr).await?;

        let num_bytes = self.read(RX_NB_BYTES).await?;
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
        let snr = self.last_packet_snr().await?;
        let rssi = self.last_packet_rssi().await?;
        Ok(RxPacket::new(coding_rate, buffer, rssi, snr))
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

    /// Gets the modem rx status.
    ///
    /// See: datasheet section 2.0.2
    pub async fn rx_status(&mut self) -> Result<RxStatus, Sx127xError<SPI::Error>> {
        Ok(RxStatus::try_from(self.read(MODEM_STAT).await? & MODEM_STAT_MODEM_STATUS_MASK).map_err(|_| InvalidState)?)
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

    /// Sets the gain and high frequency boost for the low noise receiver amplifier (LNA).
    ///
    /// See: datasheet page 110
    pub async fn set_lna(&mut self, lna: LNA) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(LNA).await?;
        set_bits(&mut byte, lna.boost_hf as u8, LNA_BOOST_HF_MASK, LNA_BOOST_HF_OFFSET);
        if lna.gain == LNAGain::Auto {
            self.set_agc_auto(true).await?;
        } else {
            self.set_agc_auto(false).await?;
            set_bits(&mut byte, lna.gain as u8, LNA_GAIN_MASK, LNA_GAIN_OFFSET);
        }
        self.write(LNA, byte).await
    }

    /// Starts the Channel Activity Detector (CAD).
    pub async fn start_cad(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::CAD).await
    }

    /// Calculates the symbol rate in chips/s.
    pub async fn symbol_rate(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let bandwidth = self.bandwidth().await?;
        let spreading_factor = self.spreading_factor().await?;

        Ok(calculate::symbol_rate(bandwidth.hz(), spreading_factor as u32) as u16)
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

    /// Unmasks an interrupt.
    ///
    /// See: datasheet section 4.1.2.4
    pub async fn unmask_interrupt<I: IRQ>(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte & !<I as IRQ>::MASK).await
    }

    /// Gets the number of valid headers received since last transition into Rx mode. Counter is reset in Sleep mode.
    pub async fn valid_rx_headers(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let msb = self.read(RX_HEADER_CNT_VALUE_MSB).await? as u16;
        let lsb = self.read(RX_HEADER_CNT_VALUE_LSB).await? as u16;
        Ok((msb << 8) | lsb)
    }

    /// Gets the number of valid packets received since last transition into Rx mode. Counter is reset in Sleep mode.
    pub async fn valid_rx_packets(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let msb = self.read(RX_PACKET_CNT_VALUE_MSB).await? as u16;
        let lsb = self.read(RX_PACKET_CNT_VALUE_LSB).await? as u16;
        Ok((msb << 8) | lsb)
    }

    // PRIVATE -------------------------------------------------------------------------------------

    /// Gets the bandwidth.
    ///
    /// See: datasheet section 4.1.1.4
    async fn bandwidth(&mut self) -> Result<Bandwidth, Sx127xError<SPI::Error>> {
        Ok(Bandwidth::from((self.read(MODEM_CONFIG_1).await? & MODEM_CONFIG_1_BW_MASK) >> MODEM_CONFIG_1_BW_OFFSET))
    }

    /// Gets the cyclic error coding rate (CR).
    ///
    /// See: datasheet section 4.1.1.3
    async fn coding_rate(&mut self) -> Result<CodingRate, Sx127xError<SPI::Error>> {
        Ok(CodingRate::from(get_bits(self.read(MODEM_CONFIG_1).await?, MODEM_CONFIG_1_CODING_RATE_MASK, MODEM_CONFIG_1_CODING_RATE_OFFSET)))
    }

    /// Configures the LoRa driver.
    async fn config(&mut self, config: Sx127xLoraConfig) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_bandwidth(config.bandwidth).await?;
        self.set_coding_rate(config.coding_rate).await?;
        self.set_frequency(config.frequency).await?;
        self.set_header_mode(config.header_mode).await?;
        self.set_spreading_factor(config.spreading_factor).await?;
        self.set_sync_word(config.sync_word).await?;
        self.set_auto_temp_calibration(config.use_auto_temp_calibration).await?;
        self.set_crc(config.use_crc).await?;

        if config.frequency != DEFAULT_FREQUENCY_HZ {
            self.calibrate().await?;
        }
        Ok(())
    }

    /// Gets the header mode.
    ///
    /// See: datasheet section 4.1.1.6
    async fn header_mode(&mut self) -> Result<HeaderMode, Sx127xError<SPI::Error>> {
        Ok(HeaderMode::from(get_bits(self.read(MODEM_CONFIG_1).await?, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_OFFSET)))
    }

    /// Maps a signal to a DIO pin.
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

    /// Performs a SPI read.
    async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(addr).await
    }

    /// Sets the automatic gain control (AGC) on/off. Turning this on will drive the LNA gain by the AGC loop as opposed to the configured LnaGain.
    ///
    /// See: datasheet table 24
    async fn set_agc_auto(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_3).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_3_AGC_AUTO_ON_MASK, MODEM_CONFIG_3_AGC_AUTO_ON_OFFSET);
        self.write(MODEM_CONFIG_3, byte).await
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

    async fn set_ocp(&mut self, ocp: OCP) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = ocp.trim();
        set_bits(&mut byte, ocp.on as u8, OCP_ON_MASK, OCP_ON_OFFSET);
        self.write(OCP, byte).await
    }

    /// Optimize receiver response for spurious reception of LoRa signal.
    ///
    /// See: errata section 2.3
    async fn set_optimize_rx_response(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        // TODO handle off
        if on {
            match self.bandwidth().await? {
                Bandwidth::Bw7_8kHz => self.optimize_rx_response(false, 0x48, 0x00, Some(7_810)).await,
                Bandwidth::Bw10_4kHz => self.optimize_rx_response(false, 0x44, 0x00, Some(10_420)).await,
                Bandwidth::Bw15_6kHz => self.optimize_rx_response(false, 0x44, 0x00, Some(15_620)).await,
                Bandwidth::Bw20_8kHz => self.optimize_rx_response(false, 0x44, 0x00, Some(20_830)).await,
                Bandwidth::Bw31_25kHz => self.optimize_rx_response(false, 0x44, 0x00, Some(31_250)).await,
                Bandwidth::Bw41_7kHz => self.optimize_rx_response(false, 0x44, 0x00, Some(41_670)).await,
                Bandwidth::Bw62_5kHz => self.optimize_rx_response(false, 0x40, 0x00, None).await,
                Bandwidth::Bw125kHz => self.optimize_rx_response(false, 0x40, 0x00, None).await,
                Bandwidth::Bw250kHz => self.optimize_rx_response(false, 0x40, 0x00, None).await,
                Bandwidth::Bw500kHz => self.set_automatic_if(true).await,
            }
        } else {
            Ok(())
        }
    }

    async fn optimize_rx_response(&mut self, automatic_if: bool, if_freq_2: u8, if_freq_1: u8, frequency_offset: Option<i32>) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;

        if let Some(offset) = frequency_offset {
            let mut frequency = self.frequency().await?;
            frequency = (frequency as i32 + offset) as u32;
            self.set_frequency(frequency).await?;
        }

        self.set_automatic_if(automatic_if).await?;
        self.write(IF_FREQ_2, if_freq_2).await?; // 0x2f
        self.write(IF_FREQ_1, if_freq_1).await // 0x30
    }

    async fn set_automatic_if(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_3).await?;
        set_bits(&mut byte, on as u8, DETECT_OPTIMIZE_AUTOMATIC_IF_ON_MASK, DETECT_OPTIMIZE_AUTOMATIC_IF_ON_OFFSET);
        self.write(MODEM_CONFIG_3, byte).await
    }

    /// Sets the rise/fall time of the power amplifier (PA).
    async fn set_power_ramp(&mut self, pa_ramp: PowerRamp) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(PA_RAMP).await?;
        self.write(PA_RAMP, byte | pa_ramp as u8).await
    }

    /// Sets the preamble length, minus 4 symbols of fixed overhead. A `length` of 6, which is the
    /// minimum valid preamble length, will yield a total of 10 symbols, and a `length` of 65535
    /// will yield a total of 65539 symbols.
    ///
    /// See: datasheet section 4.1.1.6
    async fn set_preamble_length(&mut self, preamble_length: PreambleLength) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(PREAMBLE_MSB, (preamble_length.0 >> 8) as u8).await?;
        self.write(PREAMBLE_LSB, (preamble_length.0 & 0xff) as u8).await
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

    /// Sets the LoRa sync word.
    async fn set_sync_word(&mut self, sync_word: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(SYNC_WORD, sync_word).await
    }

    /// Sets the invert I and Q signals in the Rx or TX path.
    async fn set_invert_iq(&mut self, on: bool, mask: u8, offset: u8) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(INVERT_IQ).await?;
        set_bits(&mut byte, on as u8, mask, offset);
        self.write(INVERT_IQ, byte).await?;

        // optimize
        self.write(INVERT_IQ_2, if on { INVERT_IQ_2_ON } else { INVERT_IQ_2_OFF }).await
    }

    /// Determines if low data rate optimization is necessary.
    ///
    /// See: datasheet page 31 section Low Data Rate Optimization
    async fn should_optimize_low_data_rate(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        Ok(
            check::should_optimize_for_low_data_rate(
                calculate::symbol_period(
                    calculate::symbol_rate(
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

    /// Sets the active modem.
    async fn set_modem(&mut self, modem: Modem) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, modem as u8, OP_MODE_LONG_RANGE_MODE_MASK, OP_MODE_LONG_RANGE_MODE_OFFSET);
        self.write(OP_MODE, byte).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }

    /// Performs a SPI write.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(addr, data).await
    }
}