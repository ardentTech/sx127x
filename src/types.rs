use crate::registers::RegModemStat;
use crate::types::DeviceMode::{Cad, Fsrx, Fstx, RxContinuous, RxSingle, Sleep, Stdby, Tx};

#[derive(PartialEq)]
pub enum Bandwidth {
    Bw7_8kHz = 0x0,
    Bw10_4kHz = 0x1,
    Bw15_6kHz = 0x2,
    Bw20_8kHz = 0x3,
    Bw31_25kHz = 0x4,
    Bw41_7kHz = 0x5,
    Bw62_5kHz = 0x6,
    Bw125kHz = 0x7,
    Bw250kHz = 0x8,
    Bw500kHz = 0x9,
}

pub enum CyclicErrorCoding {
    Rate4_5 = 0x1,
    Rate4_6 = 0x2,
    Rate4_7 = 0x3,
    Rate4_8 = 0x4,
}

// see: [table 16]
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
            0x0 => Sleep,
            0x1 => Stdby,
            0x2 => Fstx,
            0x3 => Tx,
            0x4 => Fsrx,
            0x5 => RxContinuous,
            0x6 => RxSingle,
            0x7 => Cad,
            _ => unreachable!()
        }
    }
    pub(crate) const fn into_bits(self) -> u8 { self as u8 }
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
impl From<RegModemStat> for RxStatus {
    fn from(value: RegModemStat) -> Self {
        if value.modem_clear() {
            RxStatus::ModemClear
        } else if value.header_info_valid() {
            RxStatus::HeaderInfoValid
        } else if value.rx_on_going() {
            RxStatus::RxOnGoing
        } else if value.signal_synchronized() {
            RxStatus::SignalSynchronized
        } else if value.signal_detected() {
            RxStatus::SignalDetected
        } else {
            RxStatus::Unknown
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpreadingFactor {
    Factor6 = 0x6,
    Factor7 = 0x7,
    Factor8 = 0x8,
    Factor9 = 0x9,
    Factor10 = 0xa,
    Factor11 = 0xb,
    Factor12 = 0xc,
}