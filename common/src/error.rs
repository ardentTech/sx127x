#[derive(Debug)]
pub enum Sx127xError<SPI> {
    InvalidInput,
    InvalidPayloadLength,
    InvalidState,
    PacketTermination,
    SPI(SPI),
}