use embedded_hal_async::spi::SpiDevice;

#[derive(Debug)]
pub enum Sx127xError<SPI> {
    SPI(SPI),
}

/// Sx127x core driver.
pub struct Sx127x<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice>Sx127x<SPI> {
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Reads the byte from the register at `addr`.
    pub async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        let mut read = [0; 2];
        // 1 wnr bit (0 for read) + 7 bit addr
        let write = [addr & 0x7f, 0];
        self.spi.transfer(&mut read, &write).await.map_err(Sx127xError::SPI)?;
        Ok(read[1])
    }

    /// Writes the `data` raw byte to the register at `addr`.
    pub async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        // 1 wnr bit (1 for write) + 7 bit addr
        let buf = [addr | 0x80, data];
        self.spi.write(&buf).await.map_err(Sx127xError::SPI)
    }
}