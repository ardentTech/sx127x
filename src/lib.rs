#![no_std]

#[cfg(feature = "defmt")]
use defmt::debug;

#[cfg(feature = "sync")]
use embedded_hal::spi::SpiDevice;

#[cfg(not(feature = "sync"))]
use embedded_hal_async::spi::SpiDevice;

use sx127x_common::error::Sx127xError;
use sx127xfsk::driver::Sx127xFsk;
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};

pub struct Sx127x<SPI> {
    fsk: Option<Sx127xFsk<SPI>>,
    lora: Option<Sx127xLora<SPI>>
}

impl<SPI: SpiDevice> Sx127x<SPI> {

    /// Initializes a new instance of the Sx127x driver with the FSK modem active.
    #[maybe_async::maybe_async]
    pub async fn new_fsk(spi: SPI) -> Result<Sx127x<SPI>, Sx127xError<SPI::Error>> {
        Ok(Self {
            fsk: Some(Sx127xFsk::new(spi).await?),
            lora: None
        })
    }

    /// Initializes a new instance of the Sx127x driver with the LoRa modem active.
    #[maybe_async::maybe_async]
    pub async fn new_lora(spi: SPI, config: Sx127xLoraConfig) -> Result<Sx127x<SPI>, Sx127xError<SPI::Error>> {
        Ok(Self {
            fsk: None,
            lora: Some(Sx127xLora::new(spi, config).await?)
        })
    }

    /// Switches the FSK modem if the LoRa modem is active.
    #[maybe_async::maybe_async]
    pub async fn switch_to_fsk(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        if let Some(lora) = self.lora.take() {
            // TODO add getter to avoid spi.spi
            self.fsk = Some(Sx127xFsk::new(lora.spi.spi).await?);
        }
        Ok(())
    }

    /// Switches the LoRa modem if the FSK modem is active.
    #[maybe_async::maybe_async]
    pub async fn switch_to_lora(&mut self, config: Sx127xLoraConfig) -> Result<(), Sx127xError<SPI::Error>> {
        if let Some(fsk) = self.fsk.take() {
            // TODO add getter to avoid spi.spi
            self.lora = Some(Sx127xLora::new(fsk.spi.spi, Sx127xLoraConfig::default()).await?);
        }
        Ok(())
    }
}