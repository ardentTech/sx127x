#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bandwidth {
    Bw7_8kHz = 0x0,
    Bw10_4kHz = 0x1,
    Bw15_6kHz = 0x2,
    Bw20_8kHz = 0x3,
    Bw31_25kHz = 0x4,
    Bw41_7kHz = 0x5,
    Bw62_5kHz = 0x6,
    #[default]
    Bw125kHz = 0x7,
    Bw250kHz = 0x8,
    Bw500kHz = 0x9,
}
impl From<u8> for Bandwidth {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Bandwidth::Bw7_8kHz,
            0x1 => Bandwidth::Bw10_4kHz,
            0x2 => Bandwidth::Bw15_6kHz,
            0x3 => Bandwidth::Bw20_8kHz,
            0x4 => Bandwidth::Bw31_25kHz,
            0x5 => Bandwidth::Bw41_7kHz,
            0x6 => Bandwidth::Bw62_5kHz,
            0x7 => Bandwidth::Bw125kHz,
            0x8 => Bandwidth::Bw250kHz,
            _ => Bandwidth::Bw500kHz,
        }
    }
}
impl Bandwidth {
    pub(crate) fn hz(&self) -> u32 {
        match self {
            Bandwidth::Bw7_8kHz => 7_800,
            Bandwidth::Bw10_4kHz => 10_400,
            Bandwidth::Bw15_6kHz => 15_600,
            Bandwidth::Bw20_8kHz => 20_800,
            Bandwidth::Bw31_25kHz => 31_250,
            Bandwidth::Bw41_7kHz => 41_700,
            Bandwidth::Bw62_5kHz => 62_500,
            Bandwidth::Bw125kHz => 125_000,
            Bandwidth::Bw250kHz => 250_000,
            _ => 500_000
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CodingRate {
    #[default]
    Cr4_5 = 0x1,
    Cr4_6 = 0x2,
    Cr4_7 = 0x3,
    Cr4_8 = 0x4,
}
impl From<u8> for CodingRate {
    fn from(value: u8) -> Self {
        match value {
            0x1 => CodingRate::Cr4_5,
            0x2 => CodingRate::Cr4_6,
            0x3 => CodingRate::Cr4_7,
            _ => CodingRate::Cr4_8,
        }
    }
}
impl Into<f32> for CodingRate {
    fn into(self) -> f32 {
        4f32 / (match self {
            CodingRate::Cr4_5 => 5f32,
            CodingRate::Cr4_6 => 6f32,
            CodingRate::Cr4_7 => 7f32,
            CodingRate::Cr4_8 => 8f32,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeviceMode {
    SLEEP = 0x0,
    STDBY = 0x1,
    FSTX = 0x2,
    TX = 0x3,
    FSRX = 0x4,
    RXCONTINUOUS = 0x5,
    RXSINGLE = 0x6,
    CAD = 0x7
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio0Signal {
    #[default]
    RxDone = 0x0,
    TxDone = 0x1,
    CadDone = 0x2,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio1Signal {
    #[default]
    RxTimeout = 0x0,
    FhssChangeChannel = 0x1,
    CadDetected = 0x2,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interrupt {
    CadDetected = 0x0,
    FhssChangeChannel = 0x1,
    CadDone = 0x2,
    TxDone = 0x3,
    ValidHeader = 0x4,
    PayloadCrcError = 0x5,
    RxDone = 0x6,
    RxTimeout = 0x7,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SpreadingFactor {
    /// Only implicit header mode is possible with Sf6.
    Sf6 = 0x6,
    #[default]
    Sf7 = 0x7,
    Sf8 = 0x8,
    Sf9 = 0x9,
    Sf10 = 0xa,
    Sf11 = 0xb,
    Sf12 = 0xc,
}
impl From<u8> for SpreadingFactor {
    fn from(value: u8) -> Self {
        match value {
            0x6 => SpreadingFactor::Sf6,
            0x7 => SpreadingFactor::Sf7,
            0x8 => SpreadingFactor::Sf8,
            0x9 => SpreadingFactor::Sf9,
            0xa => SpreadingFactor::Sf10,
            0xb => SpreadingFactor::Sf11,
            _ => SpreadingFactor::Sf12,
        }
    }
}