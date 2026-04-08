use embedded_hal_async::spi::SpiDevice;
use crate::shared::interface::Sx127xSpi;

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
        Self { spi: Sx127xSpi::new(spi) };
    }
}