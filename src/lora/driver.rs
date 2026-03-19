use embedded_hal_async::spi::SpiDevice;
use crate::common::{calculate_frf, Sx127xSpi};
use crate::lora::registers::*;
use crate::lora::types::DeviceMode;

const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;

#[derive(Debug)]
pub enum Sx127xLoraError<SPI> {
    SPI(SPI),
}

pub struct Sx127xConfig {
    pub frequency: u32, // Hz
}
impl Default for Sx127xConfig {
    fn default() -> Self {
        Self {
            frequency: DEFAULT_FREQUENCY_HZ,
        }
    }
}
impl Sx127xConfig {
    pub fn new(frequency: u32) -> Self {
        Self { frequency }
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

        driver.disable_temp_monitor().await?;
        driver.set_long_range_mode(true).await?;
        Ok(driver)
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

    // Reads from register `addr` over SPI.
    async fn read(&mut self, addr: u8) -> Result<u8, Sx127xLoraError<SPI::Error>> {
        self.spi.read(addr).await.map_err(Sx127xLoraError::SPI)
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

    // Writes `data` to register `addr` over SPI.
    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xLoraError<SPI::Error>> {
        self.spi.write(addr, data).await.map_err(Sx127xLoraError::SPI)
    }
}