#[derive(Debug)]
pub enum Sx127xError<SPI> {
    InvalidFdev,
    InvalidPayloadLength,
    InvalidPreambleLength,
    InvalidState,
    InvalidSymbolTimeout,
    PacketTermination,
    SPI(SPI),
}