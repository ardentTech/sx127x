#[cfg(feature = "defmt")]
use defmt::debug;

use embedded_hal_async::spi::SpiDevice;
use crate::lora::bits::{get_bits, set_bits};
use crate::lora::registers::*;
use crate::lora::types::*;
use crate::lora::calculate;

const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;
const FXOSC_HZ: u32 = 32_000_000;
const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);
const PAYLOAD_SIZE: usize = 255;
pub const RX_TIMEOUT_MIN_SYMBOLS: u16 = 4;
pub const RX_TIMEOUT_MAX_SYMBOLS: u16 = 1023;

#[derive(Debug)]
pub enum Sx127xLoraError<SPI> {
    InvalidPayloadLength,
    InvalidPreambleLength,
    InvalidState,
    InvalidSymbolTimeout,
    PacketTermination,
    SPI(SPI),
}

pub struct Sx127xConfig {
    pub bandwidth: Bandwidth,
    pub coding_rate: CodingRate,
    pub frequency: u32, // Hz
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

/// Sx127x driver with LoRa modem.
pub struct Sx127x<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice>Sx127x<SPI> {
    pub async fn new(spi: SPI, config: Sx127xConfig) -> Result<Sx127x<SPI>, Sx127xLoraError<SPI::Error>> {
        let mut driver = Self { spi };
        driver.set_long_range_mode(true).await?;

        driver.set_bandwidth(config.bandwidth).await?;
        driver.set_coding_rate(config.coding_rate).await?;
        driver.set_frequency(config.frequency).await?;
        driver.set_spreading_factor(config.spreading_factor).await?;

        Ok(driver)
    }

    pub async fn bandwidth(&mut self) -> Result<Bandwidth, Sx127xLoraError<SPI::Error>> {
        let modem_config_1 = self.read(MODEM_CONFIG_1).await?;
        Ok(Bandwidth::from((modem_config_1 & MODEM_CONFIG_1_BW_MASK) >> 4))
    }

    /// Triggers the IQ and RSSI calibration when set in Standby mode. Takes ~10ms.
    pub async fn calibrate(&mut self) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= 0x40;
        self.write(IMAGE_CAL, image_cal).await
    }

    /// Clears an interrupt.
    pub async fn clear_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS).await?;
        self.write(IRQ_FLAGS, byte | interrupt.mask()).await
    }

    /// Gets the cyclic error coding rate.
    pub async fn coding_rate(&mut self) -> Result<CodingRate, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(CodingRate::from(get_bits(byte, MODEM_CONFIG_1_CODING_RATE_MASK, 1)))
    }

    /// Calculates the current data rate in bits/s.
    pub async fn data_rate(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let coding_rate: f32 = self.coding_rate().await?.into();
        let symbol_rate = self.symbol_rate().await? as f32;
        let spreading_factor = (self.spreading_factor().await? as u8) as f32;
        Ok(calculate::data_rate(symbol_rate, spreading_factor, coding_rate))
    }

    /// Reads the carrier frequency.
    pub async fn frequency(&mut self) -> Result<u32, Sx127xLoraError<SPI::Error>> {
        let msb = self.read(FRF_MSB).await? as u32;
        let mid = self.read(FRF_MID).await? as u32;
        let lsb = self.read(FRF_LSB).await? as u32;
        Ok((msb << 16) | (mid << 8) | lsb)
    }

    /// Reads the frequency error indication (FEI) in Hz.
    ///
    /// See: datasheet section 4.1.5
    pub async fn frequency_error_indication_hz(&mut self) -> Result<f64, Sx127xLoraError<SPI::Error>> {
        let msb = self.read(FEI_MSB).await?;
        let mid = self.read(FEI_MID).await?;
        let lsb = self.read(FEI_LSB).await?;
        let fei = (((msb as u32) << 16) | ((mid as u32) << 8) | (lsb as u32)) as i32;
        let bandwidth = self.bandwidth().await?;

        Ok(calculate::fei_hz(fei, bandwidth.khz()))
    }

    /// Reads the frequency error indication (FEI) in PPM.
    ///
    /// See: datasheet section 4.1.5
    pub async fn frequency_error_indication_ppm(&mut self) -> Result<f64, Sx127xLoraError<SPI::Error>> {
        let hz = self.frequency_error_indication_hz().await?;
        let frf = self.frequency().await?;
        Ok(calculate::fei_ppm(hz, frf))
    }

    /// Sets the DIO0 pin signal source.
    pub async fn set_dio0(&mut self, signal: Dio0Signal) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_dio_mapping1(signal as u8, DIO_MAPPING_1_DIO0_MASK, DIO_MAPPING_1_DIO0_SHIFT).await
    }

    /// Sets the DIO1 pin signal source.
    pub async fn set_dio1(&mut self, signal: Dio1Signal) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_dio_mapping1(signal as u8, DIO_MAPPING_1_DIO1_MASK, DIO_MAPPING_1_DIO1_SHIFT).await
    }

    /// Reads received signal strength indicator (RSSI) of last packet received.
    ///
    /// See: datasheet section 3.5.5
    pub async fn last_packet_rssi(&mut self) -> Result<u8, Sx127xLoraError<SPI::Error>> {
        self.read(PKT_RSSI_VALUE).await
    }

    /// Reads estimation of signal-to-noise ratio (SNR) in dB on last packet received.
    ///
    /// See: datasheet section 3.5.5
    pub async fn last_packet_snr(&mut self) -> Result<i8, Sx127xLoraError<SPI::Error>> {
        Ok(self.read(PKT_SNR_VALUE).await? as i8 >> 2)
    }

    /// Masks an interrupt.
    pub async fn mask_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte | interrupt.mask()).await
    }

    /// Gets the modem status.
    ///
    /// See: datasheet section 2.0.2
    pub async fn modem_status(&mut self) -> Result<ModemStatus, Sx127xLoraError<SPI::Error>> {
        let byte = self.read(MODEM_STAT).await?;
        Ok(ModemStatus::from(byte))
    }

    /// Reads the byte from the register at `addr` over SPI.
    pub async fn read(&mut self, addr: u8) -> Result<u8, Sx127xLoraError<SPI::Error>> {
        let mut read = [0; 2];
        // 1 wnr bit (0 for read) + 7 bit addr
        let write = [addr & 0x7f, 0];
        self.spi.transfer(&mut read, &write).await.map_err(Sx127xLoraError::SPI)?;
        Ok(read[1])
    }

    /// Reads 255 bytes from the FIFO buffer.
    ///
    /// See: datasheet figure 10
    pub async fn read_rx_data(&mut self) -> Result<[u8; PAYLOAD_SIZE], Sx127xLoraError<SPI::Error>> {
        let reg_hop_channel = self.read(HOP_CHANNEL).await?;
        let crc_on_payload = get_bits(reg_hop_channel, HOP_CHANNEL_CRC_ON_PAYLOAD_MASK, 6) == 1;

        let irq_flags_bits = self.read(IRQ_FLAGS).await? >> 4;
        let rx_packet_termination_ok = if crc_on_payload {
            irq_flags_bits & 0xf == 0
        } else {
            irq_flags_bits & 0xc == 0 && irq_flags_bits & 0x1 == 0
        };
        if !rx_packet_termination_ok {
            return Err(Sx127xLoraError::PacketTermination)
        }

        let rx_fifo_addr = self.read(FIFO_RX_CURRENT_ADDR).await?;
        self.write(FIFO_ADDR_PTR, rx_fifo_addr).await?;
        let num_bytes = self.read(RX_NB_BYTES).await?;
        let mut buffer = [0; PAYLOAD_SIZE];
        for i in 0..num_bytes {
            let byte = self.read(FIFO).await?;
            buffer[i as usize] = byte;
        }
        Ok(buffer)
    }

    /// Enables receive mode and searches for a preamble. If a `timeout` is specified, the device
    /// enter RXSINGLE mode, else RXCONTINUOUS mode.
    ///
    /// See: datasheet pages 40-42
    pub async fn receive(&mut self, timeout: Option<u16>) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut mode = DeviceMode::RXCONTINUOUS;

        if let Some(timeout) = timeout {
            if timeout < RX_TIMEOUT_MIN_SYMBOLS || timeout > RX_TIMEOUT_MAX_SYMBOLS {
                return Err(Sx127xLoraError::InvalidSymbolTimeout)
            }
            mode = DeviceMode::RXSINGLE;

            let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
            set_bits(&mut modem_config_2, (timeout >> 8) as u8, MODEM_CONFIG_2_SYMB_TIMEOUT_MASK, 0);
            self.write(MODEM_CONFIG_2, modem_config_2).await?;

            self.write(SYMB_TIMEOUT_LSB, (timeout & 0xff) as u8).await?;
        }

        self.write(FIFO_ADDR_PTR, FIFO_RX_BASE_ADDR).await?;
        self.set_device_mode(mode).await
    }

    /// Reads the current received signal strength indicator (RSSI).
    ///
    /// See: datasheet section 3.5.5
    pub async fn rssi(&mut self) -> Result<u8, Sx127xLoraError<SPI::Error>> {
        self.read(RSSI_VALUE).await
    }

    /// Sets the bandwidth.
    ///
    /// See: datasheet section 4.1.1.4
    pub async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, bandwidth as u8, MODEM_CONFIG_1_BW_MASK, 4);

        // TODO if bandwidth == Bandwidth::Bw500kHz {
        //     self.optimize_500khz_bandwidth().await?;
        // }

        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the cyclic error coding rate.
    ///
    /// See: datasheet section 4.1.1.3
    pub async fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, coding_rate as u8, MODEM_CONFIG_1_CODING_RATE_MASK, 1);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets CRC generation and check on payload on/off.
    ///
    /// See: section 4.1.1.6
    pub async fn set_crc(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_MASK, 2);
        self.write(MODEM_CONFIG_2, byte).await
    }

    /// Sets the device mode.
    ///
    /// See: datasheet section 2.1.4
    pub async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, 0);
        self.write(OP_MODE, byte).await
    }

    /// Sets the carrier frequency. It's imperative that you check regulations for your area (e.g.
    /// 902-928 MHz for the United States)
    pub async fn set_frequency(&mut self, hz: u32) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let frf = calculate::frf(hz, FSTEP);
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

    /// Sets invert IQ config for the rx_path and tx_path.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_invert_iq(&mut self, rx_path: bool, tx_path: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(INVERT_IQ).await?;
        set_bits(&mut byte, rx_path as u8, INVERT_IQ_RX_MASK, 6);
        set_bits(&mut byte, tx_path as u8, INVERT_IQ_TX_MASK, 0);
        self.write(INVERT_IQ, byte).await
    }

    /// Sets the gain for the low noise receiver amplifier (LNA).
    ///
    /// See: datasheet page 110
    pub async fn set_lna_gain(&mut self, gain: LnaGain) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(LNA).await?;
        set_bits(&mut byte, gain as u8, LNA_GAIN_MASK, 5);
        self.write(LNA, byte).await
    }

    /// Sets the low data rate optimization.Its use is mandated when the symbol duration exceeds
    /// 16ms.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn set_low_data_rate_optimize(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_3).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_MASK, 3);
        self.write(MODEM_CONFIG_3, byte).await
    }

    /// Sets the over-current protection (OCP) on/off.
    ///
    /// See: datasheet section 3.4.4
    pub async fn set_ocp(&mut self, on: bool, imax: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let trim = if imax < 130 {
            (imax - 45) / 5
        } else {
            (imax + 30) / 10
        };
        self.write(OCP, ((on as u8) << 5) | trim).await
    }

    /// Sets the power amplifier (PA) to PA_HP on the PA_BOOST pin.
    ///
    /// See: datasheet section 3.4.2
    ///
    /// Arguments:
    ///
    /// * `power`: 2 <= a <= 17 for continuous operation, or 20 for duty-cycled operation
    pub async fn set_pa_boost(&mut self, power: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        assert!(power == 20 || (power >= 2 && power <= 17));

        let mut byte = self.read(PA_CONFIG).await?;
        set_bits(&mut byte, 1, PA_CONFIG_PA_SELECT_MASK, 7);
        set_bits(&mut byte, 7, PA_CONFIG_MAX_POWER_MASK, 4);
        set_bits(&mut byte, if power == 20 { power - 5 } else { power - 2 }, PA_CONFIG_OUTPUT_POWER_MASK, 0);
        self.write(PA_CONFIG, byte).await?;

        self.write(PA_DAC, if power == 20 { 0x87 } else { 0x84 }).await?;
        self.set_ocp(true, if power == 20 { 120 } else { 87 }).await
    }

    /// Sets the rise/fall time of the power amplifier (PA).
    pub async fn set_pa_ramp(&mut self, pa_ramp: PARamp) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(PA_RAMP).await?;
        self.write(PA_RAMP, byte | pa_ramp as u8).await
    }

    /// Sets the power amplifier (PA) to PA_HF on the RFO_HF pin or PA_LF on the RFO_LF pin.
    ///
    /// See: datasheet section 3.4.2
    ///
    /// Arguments:
    ///
    /// * `power`: -4 <= a <= 15
    pub async fn set_pa_rfo(&mut self, power: i8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        assert!(power >= -4 && power <= 15);

        let mut byte = self.read(PA_CONFIG).await?;
        set_bits(&mut byte, 0, PA_CONFIG_PA_SELECT_MASK, 7);
        if (-4..=0).contains(&power) {
            set_bits(&mut byte, 0, PA_CONFIG_MAX_POWER_MASK, 4);
            set_bits(&mut byte, (power as f32 + 4.2) as u8, PA_CONFIG_OUTPUT_POWER_MASK, 0);
        } else {
            set_bits(&mut byte, 7, PA_CONFIG_MAX_POWER_MASK, 4);
            set_bits(&mut byte, power as u8, PA_CONFIG_OUTPUT_POWER_MASK, 0);
        }

        self.write(PA_DAC, 0x84).await?;
        self.set_ocp(false, 0).await
    }

    /// Sets the preamble length, minus 4 symbols of fixed overhead. A `length` of 6, which is the
    /// minimum valid preamble length, will yield a total of 10 symbols, and a `length` of 65535
    /// will yield a total of 65539 symbols.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn set_preamble_length(&mut self, length: u16) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // TODO break out validation logic like this for unit testing
        if length < 6 {
            return Err(Sx127xLoraError::InvalidPreambleLength)
        }
        self.write(PREAMBLE_MSB, (length >> 8) as u8).await?;
        self.write(PREAMBLE_LSB, (length & 0xff) as u8).await
    }

    /// Sets the spreading factor.
    ///
    /// See: datasheet section 4.1.1.2
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
        self.write(DETECT_OPTIMIZE, detect_optimize).await
    }

    /// Sets the temperature monitor operation flag. This will switch to the FSK/OOK modem,
    /// set/unset the temp monitor flag, then switch back to the LoRa modem before returning.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_temp_monitor(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.set_long_range_mode(false).await?;

        let image_cal = self.read(IMAGE_CAL).await?;
        self.write(IMAGE_CAL, image_cal | !on as u8).await?;

        self.set_long_range_mode(true).await
    }

    /// Gets the spreading factor.
    pub async fn spreading_factor(&mut self) -> Result<SpreadingFactor, Sx127xLoraError<SPI::Error>> {
        let modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        Ok(SpreadingFactor::from(get_bits(modem_config_2, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, 4)))
    }

    /// Calculates the symbol rate in chips/s.
    pub async fn symbol_rate(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let bandwidth = self.bandwidth().await?;
        let spreading_factor = self.spreading_factor().await?;

        Ok(calculate::symbol_rate(bandwidth.hz(), spreading_factor as u32) as u16)
    }

    /// Transmits a `payload` of up to 255 bytes. Will automatically transition to STDBY when done.
    pub async fn transmit(&mut self, payload: &[u8]) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // noop if already transmitting
        if self.device_mode().await? == DeviceMode::TX {
            return Ok(())
        }

        let payload_len = payload.len();
        if payload_len > PAYLOAD_SIZE {
            return Err(Sx127xLoraError::InvalidPayloadLength);
        }

        self.set_device_mode(DeviceMode::STDBY).await?;
        self.write(FIFO_ADDR_PTR, FIFO_TX_BASE_ADDR).await?;
        for &byte in payload.iter().take(PAYLOAD_SIZE) {
            self.write(FIFO, byte).await?;
        }
        self.write(PAYLOAD_LENGTH, payload.len() as u8).await?;
        self.set_device_mode(DeviceMode::TX).await
    }

    /// Unmasks an interrupt.
    pub async fn unmask_interrupt(&mut self, interrupt: Interrupt) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte & !interrupt.mask()).await
    }

    /// Reads the number of valid headers received since last transition into Rx mode.
    pub async fn valid_rx_headers(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let msb = self.read(RX_HEADER_CNT_VALUE_MSB).await? as u16;
        let lsb = self.read(RX_HEADER_CNT_VALUE_LSB).await? as u16;
        Ok((msb << 8) | lsb)
    }

    /// Reads the number of valid packets received since last transition into Rx mode.
    pub async fn valid_rx_packets(&mut self) -> Result<u16, Sx127xLoraError<SPI::Error>> {
        let msb = self.read(RX_PACKET_CNT_VALUE_MSB).await? as u16;
        let lsb = self.read(RX_PACKET_CNT_VALUE_LSB).await? as u16;
        Ok((msb << 8) | lsb)
    }

    /// Writes the `data` byte to the register at `addr` over SPI.
    pub async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // 1 wnr bit (1 for write) + 7 bit addr
        let buf = [addr | 0x80, data];

        #[cfg(feature = "defmt")]
        debug!("writing 0b{:b} to 0x{:x}", data, addr);

        self.spi.write(&buf).await.map_err(Sx127xLoraError::SPI)
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn device_mode(&mut self) -> Result<DeviceMode, Sx127xLoraError<SPI::Error>> {
        let op_mode = self.read(OP_MODE).await?;
        Ok(DeviceMode::from(get_bits(op_mode, OP_MODE_MODE_MASK, 0)))
    }

    async fn set_dio_mapping1(&mut self, value: u8, mask: u8, left_shift: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        let mut byte = self.read(DIO_MAPPING_1).await?;
        set_bits(&mut byte, value, mask, left_shift);
        self.write(DIO_MAPPING_1, byte).await
    }

    // Selects the LoRa modem when `on` == true, and the FSK/OOK modem when `on` == false.
    async fn set_long_range_mode(&mut self, on: bool) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // also clears the FIFO buffer
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut op_mode = self.read(OP_MODE).await?;
        set_bits(&mut op_mode, on as u8, OP_MODE_LONG_RANGE_MODE_MASK, 7);
        self.write(OP_MODE, op_mode).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }
}