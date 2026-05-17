pub trait DataMode: private::Sealed {
    const DATA_MODE_BIT: u8;
}

pub enum ContinuousMode {}
impl DataMode for ContinuousMode {
    const DATA_MODE_BIT: u8 = 0x0;
}

pub enum PacketMode {}
impl DataMode for PacketMode {
    const DATA_MODE_BIT: u8 = 0x1;
}

mod private {
    use crate::data_mode::{ContinuousMode, PacketMode};

    pub trait Sealed {}
    impl Sealed for ContinuousMode {}
    impl Sealed for PacketMode {}
}