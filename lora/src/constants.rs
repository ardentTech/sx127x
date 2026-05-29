pub(crate) const HF_MIN_HZ: u32 = 779_000_000;
#[cfg(feature = "half_duplex")]
pub(crate) const PAYLOAD_SIZE: usize = 256;
#[cfg(not(feature = "half_duplex"))]
pub(crate) const PAYLOAD_SIZE: usize = 128;