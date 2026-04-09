use embedded_hal_async::spi::SpiDevice;
use crate::fsk::registers::RSSI_VALUE;
use crate::common::interface::Sx127xSpi;

#[derive(Debug)]
pub enum Sx127xFskError<SPI> {
    SPI(SPI),
}

/// Sx127x driver with FSK modem.
pub struct Sx127xFsk<SPI> {
    spi: Sx127xSpi<SPI>
}

impl <SPI: SpiDevice> Sx127xFsk<SPI> {
    pub async fn new(spi: SPI) -> Result<Sx127xFsk<SPI>, Sx127xFskError<SPI::Error>> {
        let mut driver = { spi: Sx127xSpi::new(spi) };
        driver.set_fsk_mode().await?;
        Ok(driver)
    }

    pub async fn rssi_value(&mut self) -> Result<u8, Sx127xFskError<SPI::Error>> {
        self.spi.read(RSSI_VALUE).await
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn set_fsk_mode(&mut self) -> Result<(), Sx127xFskError<SPI::Error>> {

    }
}