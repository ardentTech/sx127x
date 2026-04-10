use embedded_hal_async::spi::SpiDevice;
use sx127x_common::bits::{set_bits, unset_bits};
use sx127x_common::error::Sx127xError;
use sx127x_common::{FSTEP, FXOSC_HZ};
use sx127x_common::spi::Sx127xSpi;
use crate::calculate;
use crate::registers::*;
use crate::types::{Bandwidth, BwConfig, DeviceMode, ModulationType};

/// Sx127x driver with FSK modem.
pub struct Sx127xFsk<SPI> {
    pub spi: Sx127xSpi<SPI>
}
impl <SPI: SpiDevice> Sx127xFsk<SPI> {
    pub async fn new(spi: SPI) -> Result<Self, Sx127xError<SPI::Error>> {
        let mut driver = Self { spi: Sx127xSpi::new(spi) };

        driver.set_fsk_mode().await?;

        Ok(driver)
    }

    /// Sets the bandwidth for the channel filter.
    ///
    /// See: datasheet section 3.5.6
    pub async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(RX_BW).await?;
        let bw = BwConfig::from(bandwidth);
        set_bits(&mut byte, bw.exp, RX_BW_EXP_MASK, 0);
        set_bits(&mut byte, bw.mant, RX_BW_MANT_MASK, 3);
        Ok(())
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

    /// Gets the frequency deviation (fdev) in Hz.
    ///
    /// See: datasheet section 2.1.2.1
    pub async fn fdev(&mut self) -> Result<f32, Sx127xError<SPI::Error>> {
        let msb = self.spi.read(FDEV_MSB).await?;
        let lsb = self.spi.read(FDEV_LSB).await?;
        Ok(FSTEP * ((msb as u16) << 8 | lsb as u16) as f32)
    }

    /// Sets the bit rate.
    ///
    /// See: datasheet section 2.1.1
    pub async fn set_bite_rate(&mut self, rate: u16, frac: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(BITRATE_MSB, (rate >> 8) as u8).await?;
        self.spi.write(BITRATE_LSB, rate as u8).await?;
        self.spi.write(BITRATE_FRAC, frac).await
    }

    /// Sets the device mode.
    ///
    /// See: datasheet table 22
    // TODO try a trait since this is a dup of lora modem
    pub async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(OP_MODE).await?;
        set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, 0);
        self.spi.write(OP_MODE, byte).await
    }

    /// Sets the frequency deviation (fdev).
    ///
    /// See: datasheet section 2.1.2.1
    pub async fn set_fdev(&mut self, fdev_hz: u32) -> Result<(), Sx127xError<SPI::Error>> {
        if fdev_hz < 600 || fdev_hz > 200_000 {
            return Err(Sx127xError::InvalidFdev)
        }
        let fdev = (fdev_hz as f32 / FSTEP) as u16;
        self.spi.write(FDEV_MSB, (fdev >> 8) as u8).await?;
        self.spi.write(FDEV_LSB, fdev as u8).await
    }

    /// Sets the modulation type.
    pub async fn set_modulation_type(&mut self, modulation_type: ModulationType) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.spi.read(OP_MODE).await?;
        set_bits(&mut byte, modulation_type as u8, OP_MODE_MODULATION_TYPE_MASK, 5);
        self.spi.write(OP_MODE, byte).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    // Selects the LoRa modem when `on` == true, and the FSK/OOK modem when `on` == false.
    async fn set_fsk_mode(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        // TODO true? : also clears the FIFO buffer
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut op_mode = self.spi.read(OP_MODE).await?;
        unset_bits(&mut op_mode, OP_MODE_LONG_RANGE_MODE_MASK);
        self.spi.write(OP_MODE, op_mode).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }
}