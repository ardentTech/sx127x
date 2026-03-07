// TODO should all `from_bits` use `try_from` instead?

#[derive(Clone, Copy, Default, PartialEq)]
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
impl Bandwidth {
    pub(crate) const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Bandwidth::Bw7_8kHz,
            0x1 => Bandwidth::Bw10_4kHz,
            0x2 => Bandwidth::Bw15_6kHz,
            0x3 => Bandwidth::Bw20_8kHz,
            0x4 => Bandwidth::Bw31_25kHz,
            0x5 => Bandwidth::Bw41_7kHz,
            0x6 => Bandwidth::Bw62_5kHz,
            0x7 => Bandwidth::Bw125kHz,
            0x8 => Bandwidth::Bw250kHz,
            0x9 => Bandwidth::Bw500kHz,
            _ => unreachable!()
        }
    }
    pub(crate) const fn into_bits(self) -> u8 { self as u8 }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum CyclicErrorCoding {
    #[default]
    Rate4_5 = 0x1,
    Rate4_6 = 0x2,
    Rate4_7 = 0x3,
    Rate4_8 = 0x4,
}
impl CyclicErrorCoding {
    pub(crate) const fn from_bits(bits: u8) -> Self {
        match bits {
            0x1 => Self::Rate4_5,
            0x2 => Self::Rate4_6,
            0x3 => Self::Rate4_7,
            0x4 => Self::Rate4_8,
            _ => unreachable!()
        }
    }
    pub(crate) const fn into_bits(self) -> u8 { self as u8 }
}

// see: [table 16]
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DeviceMode {
    Sleep = 0x0,
    Stdby = 0x1,
    Fstx = 0x2,
    Tx = 0x3,
    Fsrx = 0x4,
    RxContinuous = 0x5,
    RxSingle = 0x6,
    Cad = 0x7
}
impl DeviceMode {
    pub(crate) const fn from_bits(bits: u8) -> Self {
        match bits {
            0x0 => Self::Sleep,
            0x1 => Self::Stdby,
            0x2 => Self::Fstx,
            0x3 => Self::Tx,
            0x4 => Self::Fsrx,
            0x5 => Self::RxContinuous,
            0x6 => Self::RxSingle,
            0x7 => Self::Cad,
            _ => unreachable!()
        }
    }
    pub(crate) const fn into_bits(self) -> u8 { self as u8 }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dio0 {
    RxDone = 0x0,
    TxDone = 0x1,
    CadDone = 0x2,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dio1 {
    RxTimeout = 0x0,
    FhssChangeChannel = 0x1,
    CadDetected = 0x2,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dio2 {
    FhssChangeChannel = 0x0,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dio3 {
    CadDone = 0x0,
    ValidHeader = 0x1,
    PayloadCrcError = 0x2,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dio4 {
    CadDetected = 0x0,
    PllLock = 0x1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dio5 {
    ModeReady = 0x0,
    ClkOut = 0x1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Interrupt {
    CadDetected,
    FhssChangeChannel,
    CadDone,
    TxDone,
    ValidHeader,
    PayloadCrcError,
    RxDone,
    RxTimeout,
}

// TODO make a struct and include coding rate of last header received?
//see: [page 111]
#[derive(Clone, Copy, PartialEq)]
pub enum RxStatus {
    ModemClear,
    HeaderInfoValid,
    RxOnGoing,
    SignalSynchronized,
    SignalDetected,
    Unknown,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum SpreadingFactor {
    Sf6 = 0x6,
    #[default]
    Sf7 = 0x7,
    Sf8 = 0x8,
    Sf9 = 0x9,
    Sf10 = 0xa,
    Sf11 = 0xb,
    Sf12 = 0xc,
}
impl SpreadingFactor {
    pub(crate) const fn from_bits(bits: u8) -> Self {
        match bits {
            0x6 => Self::Sf6,
            0x7 => Self::Sf7,
            0x8 => Self::Sf8,
            0x9 => Self::Sf9,
            0xa => Self::Sf10,
            0xb => Self::Sf11,
            0xc => Self::Sf12,
            _ => unreachable!()
        }
    }
    pub(crate) const fn into_bits(self) -> u8 { self as u8 }
}