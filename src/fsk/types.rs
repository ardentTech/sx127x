#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeviceMode {
    SLEEP = 0x0,
    STDBY = 0x1,
    FSTX = 0x2,
    TX = 0x3,
    FSRX = 0x4,
    RX = 0x5,
}
impl From<u8> for DeviceMode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => crate::lora::types::DeviceMode::SLEEP,
            0x1 => crate::lora::types::DeviceMode::STDBY,
            0x2 => crate::lora::types::DeviceMode::FSTX,
            0x3 => crate::lora::types::DeviceMode::TX,
            0x4 => crate::lora::types::DeviceMode::FSRX,
            0x5 => crate::lora::types::DeviceMode::RX
        }
    }
}