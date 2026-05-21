#[derive(Debug)]
pub enum Sx127xError<SPI> {
    InvalidInput,
    InvalidPayloadLength,
    InvalidState,
    InvalidVersion,
    PacketTermination,
    SPI(SPI),
}