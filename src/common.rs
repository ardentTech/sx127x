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

// TODO this might be LoRa only...
pub(crate) fn calculate_data_rate(
    symbol_rate: f32,
    spreading_factor: f32,
    coding_rate: f32,
) -> u16 {
    (symbol_rate * spreading_factor * coding_rate) as u16
}

// TODO this might be LoRa only...
pub(crate) fn calculate_frf(hz: u32) -> u32 {
    ((hz as f32) / FSTEP) as u32
}

// TODO this might be LoRa only...
pub(crate) fn calculate_symbol_rate(bandwidth: u32, spreading_factor: u32) -> u32 {
    bandwidth / 2u32.pow(spreading_factor)
}

pub(crate) const fn get_bits(byte: u8, mask: u8, lsb_offset: u8) -> u8 {
    (byte & mask) >> lsb_offset
}

pub(crate) const fn set_bits(byte: &mut u8, bits: u8, mask: u8, lsb_offset: u8) {
    *byte &= !mask;
    *byte |= (bits << lsb_offset) & mask
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_data_rate_ok() {
        let data_rate = calculate_data_rate(1953f32, 6f32, 0.8f32);
        assert_eq!(data_rate, 9374u16);
    }

    #[test]
    fn calculate_frf_ok() {
        let frf = calculate_frf(434_000_000);
        assert_eq!(frf, 0x6c8000);
    }

    #[test]
    fn calculate_symbol_rate_ok() {
        let bandwidth = 125_000u32;
        let spreading_factor = 7u32;
        let symbol_rate = calculate_symbol_rate(bandwidth, spreading_factor);
        assert_eq!(symbol_rate, 976u32);
    }

    #[test]
    fn get_bits_ok() {
        let byte = 0b1010_0011;
        assert_eq!(get_bits(byte, 0b111_0000, 4), 0b010);
    }

    #[test]
    fn set_bits_ok() {
        let mut byte = 0b0011_0101;
        set_bits(&mut byte, 0b101, 0b1110, 1);
        assert_eq!(byte, 0b0011_1011);
    }
}