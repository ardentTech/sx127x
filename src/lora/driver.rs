use embedded_hal_async::spi::SpiDevice;
use crate::core::{Sx127x, Sx127xError};

pub struct Sx127xLoRa<SPI> {
    core: Sx127x<SPI>
}

impl<SPI: SpiDevice> Sx127xLoRa<SPI> {
    pub fn new(spi: SPI) -> Self {
        Sx127xLoRa { core: Sx127x::new(spi) }
    }
}