#[cfg(feature = "defmt")]
use defmt::debug;

use core::marker::PhantomData;
use sx127x_common::bits::{get_bits, set_bits, unset_bits};
use sx127x_common::error::Sx127xError;
use sx127x_common::{FSTEP, FXOSC_HZ};
use sx127x_common::spi::{SpiDevice, Sx127xSpi};
use crate::{calculate, validate};
use crate::data_mode::DataMode;
use crate::dio::*;
use crate::registers::*;
use crate::types::*;

const FIFO_CAPACITY: usize = 64;

/// Sx127x driver with FSK modem.
pub struct Sx127xFsk<DM, SPI> {
    data_mode: PhantomData<DM>,
    pub spi: Sx127xSpi<SPI>
}

impl<SPI: SpiDevice> Sx127xFsk<crate::data_mode::ContinuousMode, SPI> {
    pub async fn set_dio0(&mut self, signal: ContinuousDio0Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO0_MASK, DIO_MAPPING_1_DIO0_MASK).await
    }

    pub async fn set_dio1(&mut self, signal: ContinuousDio1Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO1_MASK, DIO_MAPPING_1_DIO1_MASK).await
    }

    pub async fn set_dio2(&mut self, signal: ContinuousDio2Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO2_MASK, DIO_MAPPING_1_DIO2_MASK).await
    }

    pub async fn set_dio3(&mut self, signal: ContinuousDio3Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO3_MASK, DIO_MAPPING_1_DIO3_MASK).await
    }

    pub async fn set_dio4(&mut self, signal: ContinuousDio4Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_2, signal as u8, DIO_MAPPING_2_DIO4_MASK, DIO_MAPPING_2_DIO4_MASK).await
    }

    pub async fn set_dio5(&mut self, signal: ContinuousDio5Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_2, signal as u8, DIO_MAPPING_2_DIO5_MASK, DIO_MAPPING_2_DIO5_MASK).await
    }
}

impl<SPI: SpiDevice> Sx127xFsk<crate::data_mode::PacketMode, SPI> {
    pub async fn set_dio0(&mut self, signal: PacketDio0Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO0_MASK, DIO_MAPPING_1_DIO0_MASK).await
    }

    pub async fn set_dio1(&mut self, signal: PacketDio1Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO1_MASK, DIO_MAPPING_1_DIO1_MASK).await
    }

    pub async fn set_dio2(&mut self, signal: PacketDio2Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO2_MASK, DIO_MAPPING_1_DIO2_MASK).await
    }

    pub async fn set_dio3(&mut self, signal: PacketDio3Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO3_MASK, DIO_MAPPING_1_DIO3_MASK).await
    }

    pub async fn set_dio4(&mut self, signal: PacketDio4Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_2, signal as u8, DIO_MAPPING_2_DIO4_MASK, DIO_MAPPING_2_DIO4_MASK).await
    }

    pub async fn set_dio5(&mut self, signal: PacketDio5Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio(DIO_MAPPING_2, signal as u8, DIO_MAPPING_2_DIO5_MASK, DIO_MAPPING_2_DIO5_MASK).await
    }
}


impl<DM: DataMode, SPI: SpiDevice> Sx127xFsk<DM, SPI> {
    pub async fn new(spi: SPI) -> Result<Self, Sx127xError<SPI::Error>> {
        let mut driver = Self { data_mode: PhantomData, spi: Sx127xSpi::new(spi) };

        driver.set_fsk_mode().await?;
        driver.set_data_mode().await?;

        Ok(driver)
    }

    /// Gets the image calibration mechanism on/off.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn auto_image_calibration(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        Ok(get_bits(self.spi.read(IMAGE_CAL).await?, IMAGE_CAL_AUTO_IMAGE_CAL_ON, 7) == 1)
    }

    /// Gets the bit rate in b/s.
    ///
    /// See: datasheet section 2.1.1
    pub async fn bit_rate(&mut self) -> Result<f32, Sx127xError<SPI::Error>> {
        let msb = self.spi.read(BITRATE_MSB).await?;
        let lsb = self.spi.read(BITRATE_LSB).await?;
        let frac = self.spi.read(BITRATE_FRAC).await?;

        Ok(calculate::bit_rate(FXOSC_HZ as f32, ((msb as u16) << 8 | lsb as u16) as f32, frac as f32))
    }

    /// Gets the broadcast address used in address filtering.
    ///
    /// See: datasheet section 2.1.13.6
    pub async fn broadcast_addr(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(BROADCAST_ADRS).await
    }

    /// Trigger calibration of the RC oscillator.
    pub async fn calibrate_rc_oscillator(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.spi.read(OSC).await?;
        self.set_device_mode(DeviceMode::STDBY).await?;
        self.spi.write(OSC, byte | OSC_RC_CAL_START_MASK).await
    }

    /// Clears the AFC register set in RX mode.
    pub async fn clear_afc_register(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(AFC_FEI).await?;
        set_bits(&mut byte, 1, AFC_FEI_AFC_CLEAR_MASK, 1);
        self.spi.write(AFC_FEI, byte).await
    }

    /// Gets the frequency deviation (fdev) in Hz.
    ///
    /// See: datasheet section 2.1.2.1
    pub async fn fdev(&mut self) -> Result<f32, Sx127xError<SPI::Error>> {
        let msb = self.spi.read(FDEV_MSB).await?;
        let lsb = self.spi.read(FDEV_LSB).await?;
        Ok(FSTEP * ((msb as u16) << 8 | lsb as u16) as f32)
    }

    /// Gets the frequency error indication (FEI).
    ///
    /// See: datasheet section 2.1.3.4
    pub async fn fei(&mut self) -> Result<i16, Sx127xError<SPI::Error>> {
        let msb = self.spi.read(FEI_MSB).await?;
        let lsb = self.spi.read(FEI_LSB).await?;
        Ok(((msb as u16) << 8 | lsb as u16) as i16)
    }

    /// Gets the FIFO threshold used to trigger the FifoLevel interrupt.
    ///
    /// See: datasheet section 2.1.10
    pub async fn fifo_threshold(&mut self) -> Result<FifoThreshold, Sx127xError<SPI::Error>> {
        Ok(FifoThreshold(get_bits(self.spi.read(FIFO_THRESH).await?, FIFO_THRESH_FIFO_THRESHOLD_MASK, 0)))
    }

    /// Gets whether or not the image calibration is running.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn image_calibration_running(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        let byte = self.spi.read(IMAGE_CAL).await?;
        Ok(get_bits(byte, IMAGE_CAL_IMAGE_CAL_RUNNING_MASK, 5) == 1)
    }

    /// Gets the node address used in address filtering.
    ///
    /// See: datasheet section 2.1.13.6
    pub async fn node_addr(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(NODE_ADRS).await
    }

    /// Gets the packet mode settings.
    ///
    /// See: datasheet sections 2.1.13.2, 2.1.13.4, 2.1.13.6, 2.1.13.7
    pub async fn packet_config(&mut self) -> Result<PacketConfig, Sx127xError<SPI::Error>> {
        let config_1 = self.spi.read(PACKET_CONFIG_1).await?;
        let config_2 = self.spi.read(PACKET_CONFIG_2).await?;

        Ok(PacketConfig {
            address_filtering: AddressFiltering::from(get_bits(config_1, PACKET_CONFIG_1_ADDRESS_FILTERING_MASK, 1)),
            beacon_on: get_bits(config_2, PACKET_CONFIG_2_BEACON_ON_MASK, 3) == 1,
            crc_auto_clear_off: get_bits(config_1, PACKET_CONFIG_1_CRC_AUTO_CLEAR_OFF_MASK, 3) == 0,
            crc_on: get_bits(config_1, PACKET_CONFIG_1_CRC_ON_MASK, 1) == 1,
            crc_whitening_type: CrcWhiteningType::from(get_bits(config_1, PACKET_CONFIG_1_CRC_WHITENING_TYPE_MASK, 0)),
            dc_free: DcFree::from(get_bits(config_1, PACKET_CONFIG_1_DC_FREE_MASK, 5)),
            io_home_on: get_bits(config_2, PACKET_CONFIG_2_IO_HOME_ON_MASK, 3) == 1,
            packet_format: PacketFormat::from(get_bits(config_1, PACKET_CONFIG_1_PACKET_FORMAT_MASK, 7)),
            payload_length: (get_bits(config_2, PACKET_CONFIG_2_PAYLOAD_LENGTH_MASK, 0) as u16) << 8 | self.spi.read(PAYLOAD_LENGTH).await? as u16
        })
    }

    pub async fn packet_format(&mut self) -> Result<PacketFormat, Sx127xError<SPI::Error>> {
        Ok(PacketFormat::from(get_bits(self.spi.read(PACKET_CONFIG_1).await?, PACKET_CONFIG_1_PACKET_FORMAT_MASK, 7)))
    }

    /// Gets the payload length.
    ///
    /// See: datasheet section 2.1.13.2
    pub async fn payload_length(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let packet_config_2 = self.spi.read(PACKET_CONFIG_2).await?;
        Ok((get_bits(packet_config_2, PACKET_CONFIG_2_PAYLOAD_LENGTH_MASK, 0) as u16) << 8 | self.spi.read(PAYLOAD_LENGTH).await? as u16)
    }

    /// Gets the preamble detector configuration.
    ///
    /// See: datasheet section 2.1.3.6
    pub async fn preamble_detector(&mut self) -> Result<PreambleDetector, Sx127xError<SPI::Error>> {
        let byte = self.spi.read(PREAMBLE_DETECT).await?;
        Ok(PreambleDetector::from(byte))
    }

    /// Gets the preamble size to be sent.
    pub async fn preamble_size(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let msb = self.spi.read(PREAMBLE_MSB).await?;
        let lsb = self.spi.read(PREAMBLE_LSB).await?;
        Ok((msb as u16) << 8 | lsb as u16)
    }

    /// Triggers a manual restart of the receiver chain.
    ///
    /// See: datasheet section 2.1.5.6
    pub async fn restart_rx(&mut self, with_pll_lock: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(RX_CONFIG).await?;
        if with_pll_lock {
            set_bits(&mut byte, 1, RX_CONFIG_RESTART_WITH_PLL_LOCK_MASK, 5);
        } else {
            set_bits(&mut byte, 1, RX_CONFIG_RESTART_WITHOUT_PLL_LOCK_MASK, 6);
        }
        self.spi.write(RX_CONFIG, byte).await
    }

    /// Gets the absolute value of the received signal strength indicator (RSSI) in dBm, 0.5dB steps.
    ///
    /// See: datasheet section 3.5.4
    pub async fn rssi(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(RSSI_VALUE).await
    }

    /// Gets the sequencer transition options.
    ///
    /// See: datasheet section 2.1.8.2
    pub async fn sequencer_transitions(&mut self) -> Result<SequencerTransitions, Sx127xError<SPI::Error>> {
        let mut res = SequencerTransitions::default();
        res.set_config1(self.spi.read(SEQ_CONFIG_1).await?);
        res.set_config2(self.spi.read(SEQ_CONFIG_2).await?);
        Ok(res)
    }

    /// Sets the automatic frequency correction (AFC) value.
    ///
    /// See: datasheet section 2.1.3.5
    pub async fn set_afc(&mut self, afc: i16) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(AFC_MSB, (afc >> 8) as u8).await?;
        self.spi.write(AFC_LSB, afc as u8).await
    }

    /// Sets the automatic frequency correction (AFC) auto-clear. Only valid if AfcAutoOn bit of RegRxConfig is set.
    ///
    /// See: datasheet section 2.1.3.5
    pub async fn set_afc_auto_clear(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(AFC_FEI).await?;
        set_bits(&mut byte, on as u8, AFC_FEI_AFC_AUTO_CLEAR_ON_MASK, 0);
        self.spi.write(AFC_FEI, byte).await
    }

    /// Sets the bandwidth for automatic frequency correction (AFC).
    ///
    /// See: datasheet section 2.1.3.5
    pub async fn set_afc_bw(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(AFC_BW).await?;
        let bw = BwConfig::from(bandwidth);
        set_bits(&mut byte, bw.exp, AFC_BW_EXP_MASK, 0);
        set_bits(&mut byte, bw.mant, AFC_BW_MANT_MASK, 3);
        self.spi.write(AFC_BW, byte).await
    }

    /// Sets the automatic gain correction (AGC) threshold.
    ///
    /// TODO
    pub async fn set_agc_threshold(&mut self, threshold: AgcThreshold) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(threshold.reg_addr()).await?;
        set_bits(&mut byte, threshold.value(), threshold.mask(), threshold.offset());
        self.spi.write(threshold.reg_addr(), byte).await
    }

    /// Sets the image calibration mechanism on/off.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_auto_image_calibration(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(IMAGE_CAL).await?;
        set_bits(&mut byte, on as u8, IMAGE_CAL_AUTO_IMAGE_CAL_ON, 7);
        self.spi.write(IMAGE_CAL, byte).await
    }

    /// Sets the bit rate.
    ///
    /// See: datasheet section 2.1.1
    pub async fn set_bit_rate(&mut self, rate: u16, frac: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(BITRATE_MSB, (rate >> 8) as u8).await?;
        self.spi.write(BITRATE_LSB, rate as u8).await?;
        self.spi.write(BITRATE_FRAC, frac).await
    }

    /// Sets the broadcast address used in address filtering.
    ///
    /// See: datasheet section 2.1.13.6
    pub async fn set_broadcast_addr(&mut self, addr: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(BROADCAST_ADRS, addr).await
    }

    /// Sets the CLKOUT frequency.
    ///
    /// See: datasheet section 2.1.11
    pub async fn set_clk_out(&mut self, clk_out: ClkOut) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(OSC).await?;
        set_bits(&mut byte, clk_out as u8, OSC_CLK_OUT_MASK, 0);
        self.spi.write(OSC, byte).await
    }

    /// Sets the device mode.
    ///
    /// See: datasheet table 22
    // TODO try a trait since this is a dup of lora modem
    pub async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        #[cfg(feature = "defmt")]
        debug!("set_device_mode: {:?}", device_mode);
        
        let mut byte = self.spi.read(OP_MODE).await?;
        set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, 0);
        self.spi.write(OP_MODE, byte).await
    }

    /// Sets the fast frequency hopping mode.
    ///
    /// See: datasheet section 2.1.5.6
    pub async fn set_fast_frequency_hopping_mode(&mut self, mode: FastFrequencyHoppingMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(PLL_HOP).await?;
        set_bits(&mut byte, mode as u8, PLL_HOP_FAST_HOP_ON_MASK, 7);
        self.spi.write(PLL_HOP, byte).await
    }

    /// Sets the frequency deviation (fdev).
    ///
    /// See: datasheet section 2.1.2.1
    pub async fn set_fdev(&mut self, hz: u32) -> Result<(), Sx127xError<SPI::Error>> {
        if !validate::fdev(hz) {
            return Err(Sx127xError::InvalidInput)
        }
        let fdev = (hz as f32 / FSTEP) as u16;
        self.spi.write(FDEV_MSB, (fdev >> 8) as u8).await?;
        self.spi.write(FDEV_LSB, fdev as u8).await
    }

    /// Sets the frequency error indication (FEI).
    ///
    /// See: datasheet section 2.1.3.4
    pub async fn set_fei(&mut self, fei: i16) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(FEI_MSB, (fei >> 8) as u8).await?;
        self.spi.write(FEI_LSB, fei as u8).await
    }

    /// Sets the FIFO threshold used to trigger the FifoLevel interrupt.
    ///
    /// See: datasheet section 2.1.10
    pub async fn set_fifo_threshold(&mut self, threshold: FifoThreshold) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(FIFO_THRESH).await?;
        set_bits(&mut byte, threshold.0, FIFO_THRESH_FIFO_THRESHOLD_MASK, 0);
        self.spi.write(FIFO_THRESH, byte).await
    }

    /// Sets the carrier frequency.
    ///
    /// See: datasheet Table 32
    pub async fn set_frequency(&mut self, hz: u32) -> Result<(), Sx127xError<SPI::Error>> {
        // TODO SX1279 has more range
        // TODO getter
        let frf = sx127x_common::calculate::frf(hz, FSTEP);
        self.spi.write(FRF_MSB, (frf >> 16) as u8).await?;
        self.spi.write(FRF_MID, (frf >> 8) as u8).await?;
        self.spi.write(FRF_LSB, frf as u8).await
    }

    /// Sets an additional delay before an automatic receiver restart is launched.
    ///
    /// See: datasheet section 2.1.7.2
    pub async fn set_inter_packet_rx_delay(&mut self, delay: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(RX_DELAY, delay).await
    }

    /// Sets the low battery detector on/off.
    ///
    /// See: datasheet section 3.2
    pub async fn set_low_battery_detector(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(LOW_BAT).await?;
        set_bits(&mut byte, on as u8, LOW_BAT_ON_MASK, 3);
        self.spi.write(LOW_BAT, byte).await
    }

    /// Sets the trimming of the low battery detection threshold.
    ///
    /// See: datasheet section 3.2
    pub async fn set_low_battery_trim(&mut self, threshold: LowBatteryThreshold) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(LOW_BAT).await?;
        set_bits(&mut byte, threshold as u8, LOW_BAT_TRIM_MASK, 3);
        self.spi.write(LOW_BAT, byte).await
    }


    /// Sets the modulation type.
    pub async fn set_modulation_type(&mut self, modulation_type: ModulationType) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(OP_MODE).await?;
        set_bits(&mut byte, modulation_type as u8, OP_MODE_MODULATION_TYPE_MASK, 5);
        self.spi.write(OP_MODE, byte).await
    }

    /// Sets the node address used in address filtering.
    ///
    /// See: datasheet section 2.1.13.6
    pub async fn set_node_addr(&mut self, addr: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(NODE_ADRS, addr).await
    }

    /// Sets the average of the OOK demod config.
    ///
    /// See: datasheet section 2.1.3.2
    pub async fn set_ook_avg(&mut self, config: OokAvg) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(OOK_AVG).await?;
        set_bits(&mut byte, config.ook_peak_thresh_dec as u8, OOK_AVG_OOK_PEAK_THRESH_DEC_MASK, 5);
        set_bits(&mut byte, config.ook_average_offset as u8, OOK_AVG_OOK_AVERAGE_OFFSET, 2);
        set_bits(&mut byte, config.ook_average_thresh_filt as u8, OOK_AVG_OOK_AVERAGE_THRESH_FILT, 0);
        self.spi.write(OOK_AVG, byte).await
    }

    /// Sets the OOK peak configuration.
    ///
    /// See: datasheet section 2.1.3.2
    pub async fn set_ook_peak_config(&mut self, config: OokPeakConfig) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(OOK_PEAK).await?;
        set_bits(&mut byte, config.bit_sync_on as u8, OOK_PEAK_BIT_SYNC_ON_MASK, 5);
        set_bits(&mut byte, config.ook_thresh_type as u8, OOK_PEAK_OOK_THRESH_TYPE_MASK, 3);
        set_bits(&mut byte, config.ook_peak_thresh as u8, OOK_PEAK_OOK_PEAK_THRESH_STEP_MASK, 0);
        self.spi.write(OOK_PEAK, byte).await
    }

    /// Sets the fixed threshold for the Data Slicer in OOK mode, or the floor threshold for the Data Slicer in OOK when Peak mode is used.
    ///
    /// See: datasheet section 2.1.3.2
    pub async fn set_ook_threshold(&mut self, threshold: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(OOK_FIX, threshold).await
    }

    /// Sets packet mode settings.
    ///
    /// See: datasheet sections 2.1.13.2, 2.1.13.4, 2.1.13.6, 2.1.13.7
    pub async fn set_packet_config(&mut self, config: PacketConfig) -> Result<(), Sx127xError<SPI::Error>> {
        let mut config1 = 0u8;
        // TODO break all of these out into individual methods?
        set_bits(&mut config1, config.packet_format as u8, PACKET_CONFIG_1_PACKET_FORMAT_MASK, 7);
        set_bits(&mut config1, config.dc_free as u8, PACKET_CONFIG_1_DC_FREE_MASK, 5);
        set_bits(&mut config1, config.crc_on as u8, PACKET_CONFIG_1_CRC_ON_MASK, 4);
        set_bits(&mut config1, config.crc_auto_clear_off as u8, PACKET_CONFIG_1_CRC_AUTO_CLEAR_OFF_MASK, 3);
        set_bits(&mut config1, config.address_filtering as u8, PACKET_CONFIG_1_ADDRESS_FILTERING_MASK, 1);
        set_bits(&mut config1, config.crc_whitening_type as u8, PACKET_CONFIG_1_CRC_WHITENING_TYPE_MASK, 0);
        self.spi.write(PACKET_CONFIG_1, config1).await?;

        let mut config2 = 0u8;
        //set_bits(&mut config2, config.data_mode as u8, PACKET_CONFIG_2_DATA_MODE_MASK, 6);
        set_bits(&mut config2, config.io_home_on as u8, PACKET_CONFIG_2_IO_HOME_ON_MASK, 5);
        set_bits(&mut config2, config.beacon_on as u8, PACKET_CONFIG_2_BEACON_ON_MASK, 3);
        set_bits(&mut config2, (config.payload_length >> 8) as u8, PACKET_CONFIG_2_PAYLOAD_LENGTH_MASK, 0);
        self.spi.write(PACKET_CONFIG_2, config2).await?;

        self.spi.write(PAYLOAD_LENGTH, config.payload_length as u8).await
    }

    /// Sets the packet format.
    ///
    /// See: datasheet section 2.1.13.2
    pub async fn set_packet_format(&mut self, format: PacketFormat) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(PACKET_CONFIG_1).await?;
        set_bits(&mut byte, format as u8, PACKET_CONFIG_1_PACKET_FORMAT_MASK, 7);
        self.spi.write(PACKET_CONFIG_1, byte).await
    }

    /// Sets the payload length.
    ///
    /// See: datasheet section 2.1.13.2
    pub async fn set_payload_length(&mut self, length: u16) -> Result<(), Sx127xError<SPI::Error>> {
        let mut packet_config_2 = self.spi.read(PACKET_CONFIG_2).await?;
        set_bits(&mut packet_config_2, (length >> 8) as u8, PACKET_CONFIG_2_PAYLOAD_LENGTH_MASK, 0);
        self.spi.write(PACKET_CONFIG_2, packet_config_2).await?;
        self.spi.write(PAYLOAD_LENGTH, length as u8).await
    }

    /// Sets the preamble detector configuration.
    ///
    /// See: datasheet section 2.1.3.6
    pub async fn set_preamble_detector(&mut self, detector: PreambleDetector) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = 0u8;
        set_bits(&mut byte, detector.on as u8, PREAMBLE_DETECT_PREAMBLE_DETECTOR_ON_MASK, 7);
        set_bits(&mut byte, detector.size as u8, PREAMBLE_DETECT_PREAMBLE_DETECTOR_SIZE_MASK, 5);
        set_bits(&mut byte, detector.tolerance.0, PREAMBLE_DETECT_PREAMBLE_DETECTOR_TOL_MASK, 0);
        self.spi.write(PREAMBLE_DETECT, byte).await
    }

    /// Sets the preamble size to be sent.
    pub async fn set_preamble_size(&mut self, size: u16) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(PREAMBLE_MSB, (size >> 8) as u8).await?;
        self.spi.write(PREAMBLE_LSB, size as u8).await
    }

    /// Sets the received signal strength indicator (RSSI) collision threshold.
    ///
    /// Seee: datasheet section 2.1.7.3
    pub async fn set_rssi_collision_threshold(&mut self, db: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(RSSI_COLLISION, db).await
    }

    /// Sets the received signal strength indicator (RSSI) offset.
    ///
    /// See: datasheet section 3.5.4
    pub async fn set_rssi_offset(&mut self, offset: i8) -> Result<(), Sx127xError<SPI::Error>> {
        if !validate::rssi_offset(offset) {
            return Err(Sx127xError::InvalidInput)
        }
        let mut byte = self.spi.read(RSSI_CONFIG).await?;
        set_bits(&mut byte, offset as u8, RSSI_CONFIG_RSSI_OFFSET_MASK, 3);
        self.spi.write(RSSI_CONFIG, byte).await
    }

    /// Sets the received signal strength indicator (RSSI) smoothing.
    ///
    /// See: datasheet section 3.5.4
    pub async fn set_rssi_smoothing(&mut self, smoothing: RssiSmoothing) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(RSSI_CONFIG).await?;
        set_bits(&mut byte, smoothing as u8, RSSI_CONFIG_RSSI_SMOOTHING_MASK, 0);
        self.spi.write(RSSI_CONFIG, byte).await
    }

    /// Sets the received signal strength indicator (RSSI) trigger level for the Rssi interrupt.
    ///
    /// Seee: datasheet section 2.1.3.9
    pub async fn set_rssi_threshold(&mut self, control: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(RSSI_THRESH, control).await
    }

    /// Sets the bandwidth for the channel filter.
    ///
    /// See: datasheet section 3.5.6
    pub async fn set_rx_bw(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(RX_BW).await?;
        let bw = BwConfig::from(bandwidth);
        set_bits(&mut byte, bw.exp, RX_BW_EXP_MASK, 0);
        set_bits(&mut byte, bw.mant, RX_BW_MANT_MASK, 3);
        self.spi.write(RX_BW, byte).await
    }

    /// Sets the receiver config.
    ///
    /// See: datasheet page 96
    pub async fn set_rx_config(&mut self, config: RxConfig) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(RX_CONFIG).await?;
        set_bits(&mut byte, config.afc_auto_on as u8, RX_CONFIG_AFC_AUTO_ON_MASK, 4);
        set_bits(&mut byte, config.agc_auto_on as u8, RX_CONFIG_AGC_AUTO_ON_MASK, 3);
        set_bits(&mut byte, config.restart_rx_on_collision as u8, RX_CONFIG_RESTART_RX_ON_COLLISION_MASK, 7);
        set_bits(&mut byte, config.rx_trigger, RX_CONFIG_RX_TRIGGER_MASK, 0);
        self.spi.write(RX_CONFIG, byte).await
    }

    /// Sets the sequencer transition options.
    ///
    /// See: datasheet section 2.1.8.2
    pub async fn set_sequencer_transitions(&mut self, options: SequencerTransitions) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(SEQ_CONFIG_1).await?;
        set_bits(&mut byte, options.idle_mode as u8, SEQ_CONFIG_1_IDLE_MODE_MASK, 5);
        set_bits(&mut byte, options.from_start as u8, SEQ_CONFIG_1_FROM_START_MASK, 4);
        set_bits(&mut byte, options.low_power_selection as u8, SEQ_CONFIG_1_LOW_POWER_SELECTION_MASK, 2);
        set_bits(&mut byte, options.from_idle as u8, SEQ_CONFIG_1_FROM_IDLE_MASK, 1);
        set_bits(&mut byte, options.from_transmit as u8, SEQ_CONFIG_1_FROM_TRANSMIT_MASK, 0);
        self.spi.write(SEQ_CONFIG_1, byte).await?;

        let mut byte = self.spi.read(SEQ_CONFIG_2).await?;
        set_bits(&mut byte, options.from_receive as u8, SEQ_CONFIG_2_FROM_RECEIVE_MASK, 5);
        set_bits(&mut byte, options.from_rx_timeout as u8, SEQ_CONFIG_2_FROM_RX_TIMEOUT_MASK, 3);
        set_bits(&mut byte, options.from_packet_received as u8, SEQ_CONFIG_2_FROM_PACKET_RECEIVED, 0);
        self.spi.write(SEQ_CONFIG_2, byte).await
    }

    /// Sets the RX signal sync timeout.
    ///
    /// See: datasheet section 2.1.3.9
    pub async fn set_rx_signal_sync_timeout(&mut self, timeout: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(RX_TIMEOUT_3, timeout).await
    }

    /// Sets the sync word recognition configuration.
    ///
    /// See: datasheet sections 2.1.7.2, 2.1.10.1, 2.1.13.6
    pub async fn set_sync_config(&mut self, config: SyncConfig) -> Result<(), Sx127xError<SPI::Error>> {
        // TODO put this on config struct?
        if !validate::sync_size(config.sync_size) {
            return Err(Sx127xError::InvalidInput)
        }
        let mut byte = self.spi.read(SYNC_CONFIG).await?;
        set_bits(&mut byte, config.auto_restart_rx_mode as u8, SYNC_CONFIG_AUTO_RESTART_RX_MODE_MASK, 6);
        set_bits(&mut byte, config.preamble_polarity as u8, SYNC_CONFIG_PREAMBLE_POLARITY_MASK, 5);
        set_bits(&mut byte, config.sync_on as u8, SYNC_CONFIG_SYNC_ON_MASK, 4);
        set_bits(&mut byte, config.sync_size, SYNC_CONFIG_SYNC_SIZE_MASK, 0);
        self.spi.write(SYNC_CONFIG, byte).await
    }

    /// Sets the sync word values. `values[0]` == RegSyncValue1 (MSB byte) ... `values[7]` == RegSyncValue8.
    /// Since "SyncValue choices containing 0x00 bytes are not allowed", if 0x00 bytes in `values` will be
    /// converted to 0x01, which is the default register value.
    ///
    /// See: datasheet section 2.1.10.1
    pub async fn set_sync_values(&mut self, values: &[u8; 8]) -> Result<(), Sx127xError<SPI::Error>> {
        for (i, n) in values.iter().enumerate() {
            self.spi.write(SYNC_VALUE_1 + i as u8, if *n == 0 { 0x1 } else { *n }).await?;
        }
        Ok(())
    }

    /// Sets the temperature monitor operation on/off.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_temp_monitor(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(IMAGE_CAL).await?;
        set_bits(&mut byte, !(on as u8), IMAGE_CAL_TEMP_MONITOR_OFF, 0);
        self.spi.write(IMAGE_CAL, byte).await
    }

    /// Sets the temperature change threshold to trigger a new I/Q calibration.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_temp_threshold(&mut self, temp_threshold: TempThreshold) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(IMAGE_CAL).await?;
        set_bits(&mut byte, temp_threshold as u8, IMAGE_CAL_TEMP_THRESHOLD_MASK, 1);
        self.spi.write(IMAGE_CAL, byte).await
    }

    /// Sets the RX preamble timeout.
    ///
    /// See: datasheet section 2.1.3.9
    pub async fn set_rx_preamble_timeout(&mut self, timeout: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(RX_TIMEOUT_2, timeout).await
    }

    /// Sets the RX RSSI timeout.
    ///
    /// See: datasheet section 2.1.3.9
    pub async fn set_rx_rssi_timeout(&mut self, timeout: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(RX_TIMEOUT_1, timeout).await
    }

    /// Sets the resolution of Timer 1.
    ///
    /// See: datasheet section 2.1.8.3
    pub async fn set_timer1(&mut self, config: TimerConfig) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(TIMER_RESOL).await?;
        set_bits(&mut byte, config as u8, TIMER_RESOL_TIMER_1_RESOLUTION, 2);
        self.spi.write(TIMER_RESOL, byte).await
    }

    /// Sets the resolution of Timer 2.
    ///
    /// See: datasheet section 2.1.8.3
    pub async fn set_timer2(&mut self, config: TimerConfig) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(TIMER_RESOL).await?;
        set_bits(&mut byte, config as u8, TIMER_RESOL_TIMER_2_RESOLUTION, 0);
        self.spi.write(TIMER_RESOL, byte).await
    }

    /// Sets the coefficient for Timer1.
    ///
    /// See: datasheet section 2.1.8.3
    pub async fn set_timer1_coefficient(&mut self, coefficient: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(TIMER_1_COEFF, coefficient).await
    }

    /// Sets the coefficient for Timer2.
    ///
    /// See: datasheet section 2.1.8.3
    pub async fn set_timer2_coefficient(&mut self, coefficient: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(TIMER_2_COEFF, coefficient).await
    }

    /// Sets the condition to start packet transmission.
    ///
    /// See: datasheet section 2.1.13.3
    pub async fn set_tx_start_condition(&mut self, condition: TxStartCondition) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(FIFO_THRESH).await?;
        set_bits(&mut byte, condition as u8, FIFO_THRESH_TX_START_CONDITION_MASK, 7);
        self.spi.write(FIFO_THRESH_TX_START_CONDITION_MASK, byte).await
    }

    /// Triggers an AGC sequence.
    ///
    /// See: datasheet section 2.1.3.5
    pub async fn start_agc_sequence(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.spi.read(AFC_FEI).await?;
        self.spi.write(AFC_FEI, byte | AFC_FEI_AGC_START_MASK).await
    }

    /// Triggers the IQ and RSSI calibration when set in Standby mode.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn start_image_calibration(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut byte = self.spi.read(IMAGE_CAL).await?;
        set_bits(&mut byte, 1, IMAGE_CAL_IMAGE_CAL_START_MASK, 6);
        self.spi.write(IMAGE_CAL, byte).await
    }

    /// Starts the top level sequencer.
    ///
    /// See: datasheet section 2.1.8
    pub async fn start_sequencer(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut byte = self.spi.read(SEQ_CONFIG_1).await?;
        set_bits(&mut byte, 1, SEQ_CONFIG_1_SEQUENCER_START_MASK, 7);
        self.spi.write(SEQ_CONFIG_1, byte).await
    }

    /// Stops the top level sequencer.
    ///
    /// See: datasheet section 2.1.8
    pub async fn stop_sequencer(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(SEQ_CONFIG_1).await?;
        set_bits(&mut byte, 1, SEQ_CONFIG_1_SEQUENCER_STOP_MASK, 6);
        self.spi.write(SEQ_CONFIG_1, byte).await
    }

    /// Gets the temperature measurement.
    ///
    /// See: datasheet section 3.5.7
    pub async fn temp(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(TEMP).await
    }

    /// Gets the IRQ flag witnessing a temperature change exceeding TempThreshold since the last Image and RSSI calibration.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn temp_change_greater_than_threshold(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        let byte = self.spi.read(IMAGE_CAL).await?;
        Ok(get_bits(byte, IMAGE_CAL_TEMP_CHANGE_MASK, 3) == 1)
    }

    /// Gets the temperature monitor operation on/off.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn temp_monitor(&mut self) -> Result<bool, Sx127xError<SPI::Error>> {
        Ok(get_bits(self.spi.read(IMAGE_CAL).await?, IMAGE_CAL_TEMP_MONITOR_OFF, 0) == 0)
    }

    /// Gets the temperature change threshold to trigger a new I/Q calibration.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn temp_threshold(&mut self) -> Result<TempThreshold, Sx127xError<SPI::Error>> {
        Ok(TempThreshold::from(get_bits(self.spi.read(IMAGE_CAL).await?, IMAGE_CAL_TEMP_THRESHOLD_MASK, 1)))
    }

    /// Sets the coefficient for Timer1.
    ///
    /// See: datasheet section 2.1.8.3
    pub async fn timer1_coefficient(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(TIMER_1_COEFF).await
    }

    /// Sets the coefficient for Timer2.
    ///
    /// See: datasheet section 2.1.8.3
    pub async fn timer2_coefficient(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(TIMER_2_COEFF).await
    }

    pub async fn transmit(&mut self, payload: &[u8]) -> Result<(), Sx127xError<SPI::Error>> {
        // TODO sleep/stdby mode?

        // if fixed packet format, payload is limited to 2047 bytes
        // if len payload > FIFO_CAPACITY, loop {}
        let len = payload.len();
        if len > FIFO_CAPACITY {
            // self.set_device_mode(DeviceMode::TX).await?;
            // // TODO really need a dep just for chunks()?
            // for chunk in &payload.iter().chunks(FIFO_CAPACITY) {
            //     for &byte in chunk {
            //         self.spi.write(FIFO, byte).await?;
            //     }
            //     // TODO wait for FifoThreshold or FifoEmpty
            // }
        } else {
            for &byte in payload.iter() {
                self.spi.write(FIFO, byte).await?;
            }
            self.set_device_mode(DeviceMode::TX).await?;
        }

        // FifoFull is set when FIFO contains 66 bytes (64 + CRC?) (64 + Addr + Len?)

        Ok(())
    }

    /// Gets the condition to start packet transmission.
    ///
    /// See: datasheet section 2.1.13.3
    pub async fn tx_start_condition(&mut self) -> Result<TxStartCondition, Sx127xError<SPI::Error>> {
        Ok(TxStartCondition::from(get_bits(self.spi.read(FIFO_THRESH).await?, FIFO_THRESH_TX_START_CONDITION_MASK, 7)))
    }

    /// Gets the version code of the chip. Bits 7-4 give the full revision number; bits 3-0 give the metal mask revision number.
    pub async fn version(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(VERSION).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn set_data_mode(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(PACKET_CONFIG_2).await?;
        set_bits(&mut byte, DM::DATA_MODE_BIT, PACKET_CONFIG_2_DATA_MODE_MASK, 6);
        self.spi.write(PACKET_CONFIG_2, byte).await
    }

    async fn set_dio(&mut self, addr: u8, bits: u8, mask: u8, shift: u8) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(addr).await?;
        set_bits(&mut byte, bits, mask, shift);
        self.spi.write(addr, byte).await
    }

    // Selects the LoRa modem when `on` == true, and the FSK/OOK modem when `on` == false.
    async fn set_fsk_mode(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut op_mode = self.spi.read(OP_MODE).await?;
        unset_bits(&mut op_mode, OP_MODE_LONG_RANGE_MODE_MASK);
        self.spi.write(OP_MODE, op_mode).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }
}