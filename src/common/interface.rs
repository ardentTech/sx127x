use embedded_hal_async::spi::SpiDevice;

pub(crate) struct Sx127xSpi<SPI> {
    spi: SPI
}
impl <SPI: SpiDevice> Sx127xSpi<SPI> {
    pub(crate) fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Gets the byte from the register at `addr` over SPI.
    ///
    /// See: datasheet section 2.2
    pub(crate) async fn read(&mut self, addr: u8) -> Result<u8, SPI::Error> {
        let mut read = [0; 2];
        // 1 wnr bit (0 for read) + 7 bit addr
        let write = [addr & 0x7f, 0];
        self.spi.transfer(&mut read, &write).await?;
        Ok(read[1])
    }

    /// Writes the `data` byte to the register at `addr` over SPI.
    ///
    /// See: datasheet section 2.2
    pub(crate) async fn write(&mut self, addr: u8, data: u8) -> Result<(), SPI::Error> {
        // 1 wnr bit (1 for write) + 7 bit addr
        let buf = [addr | 0x80, data];
        self.spi.write(&buf).await
    }
}