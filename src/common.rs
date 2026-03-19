use embedded_hal_async::spi::SpiDevice;

const FXOSC_HZ: u32 = 32_000_000;
const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);

/// Sx127x core driver.
pub(crate) struct Sx127xSpi<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice> Sx127xSpi<SPI> {
    pub(crate) fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Reads the byte from the register at `addr`.
    pub(crate) async fn read(&mut self, addr: u8) -> Result<u8, SPI::Error> {
        let mut read = [0; 2];
        // 1 wnr bit (0 for read) + 7 bit addr
        let write = [addr & 0x7f, 0];
        self.spi.transfer(&mut read, &write).await?;
        Ok(read[1])
    }

    /// Writes the `data` raw byte to the register at `addr`.
    pub(crate) async fn write(&mut self, addr: u8, data: u8) -> Result<(), SPI::Error> {
        // 1 wnr bit (1 for write) + 7 bit addr
        let buf = [addr | 0x80, data];
        self.spi.write(&buf).await
    }
}

pub(crate) fn calculate_frf(hz: u32) -> u32 {
    ((hz as f32) / FSTEP) as u32
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculate_frf_ok() {
        let frf = calculate_frf(434_000_000);
        assert_eq!(frf, 0x6c8000);
    }
}