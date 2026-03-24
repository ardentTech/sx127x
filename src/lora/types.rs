use crate::lora::registers::{INVERT_IQ_RX_MASK, INVERT_IQ_TX_MASK, LNA_BOOST_HF_MASK, LNA_GAIN_MASK, OCP_ON_MASK, OCP_TRIM_MASK};

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

    pub(crate) fn khz(&self) -> f32 {
        match self {
            Bandwidth::Bw7_8kHz => 7.8,
            Bandwidth::Bw10_4kHz => 10.4,
            Bandwidth::Bw15_6kHz => 15.6,
            Bandwidth::Bw20_8kHz => 20.8,
            Bandwidth::Bw31_25kHz => 31.25,
            Bandwidth::Bw41_7kHz => 41.7,
            Bandwidth::Bw62_5kHz => 62.5,
            Bandwidth::Bw125kHz => 125.0,
            Bandwidth::Bw250kHz => 250.0,
            _ => 500.0
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
impl From<u8> for DeviceMode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => DeviceMode::SLEEP,
            0x1 => DeviceMode::STDBY,
            0x2 => DeviceMode::FSTX,
            0x3 => DeviceMode::TX,
            0x4 => DeviceMode::FSRX,
            0x5 => DeviceMode::RXCONTINUOUS,
            0x6 => DeviceMode::RXSINGLE,
            _ => DeviceMode::CAD,
        }
    }
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

pub struct FEI {
    hz: f64,
    ppm: f64,
}
impl FEI {
    pub(crate) fn new(fei: i32, bandwidth_khz: f32, frf: u32) -> Self {
        let hz: f64 = ((fei * 2i32.pow(24) / (32 * 10i32.pow(6))) as f64) * ((bandwidth_khz / 500f32) as f64);
        Self { hz, ppm: hz * (10u32.pow(6) / frf) as f64 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HeaderMode {
    #[default]
    Explicit = 0x0,
    Implicit = 0x1,
}
impl From<u8> for HeaderMode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => HeaderMode::Explicit,
            _ => HeaderMode::Implicit,
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvertIQ {
    pub rx_path: bool,
    pub tx_path: bool,
}
impl From<u8> for InvertIQ {
    fn from(value: u8) -> Self {
        Self {
            rx_path: ((value & INVERT_IQ_RX_MASK) >> 6) == 1,
            tx_path: (value & INVERT_IQ_TX_MASK) == 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LnaGain {
    #[default]
    G1 = 0x1,
    G2 = 0x2,
    G3 = 0x3,
    G4 = 0x4,
    G5 = 0x5,
    G6 = 0x6,
}
impl From<u8> for LnaGain {
    fn from(value: u8) -> Self {
        match value {
            0x1 => LnaGain::G1,
            0x2 => LnaGain::G2,
            0x3 => LnaGain::G3,
            0x4 => LnaGain::G4,
            0x5 => LnaGain::G5,
            _ => LnaGain::G6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LnaGainConfig {
    pub boost_hf: bool,
    pub gain: LnaGain,
}
impl From<u8> for LnaGainConfig {
    fn from(value: u8) -> Self {
        Self {
            boost_hf: (value & LNA_BOOST_HF_MASK) == 0x3,
            gain: LnaGain::from((value & LNA_GAIN_MASK) >> 5)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModemStatus {
    SignalDetected = 0x0,
    SignalSynchronized = 0x1,
    RxOnGoing = 0x4,
    HeaderInfoValid = 0x8,
    ModemClear = 0x16,
}
impl From<u8> for ModemStatus {
    fn from(value: u8) -> Self {
        match value {
            0x0 => ModemStatus::SignalDetected,
            0x1 => ModemStatus::SignalSynchronized,
            0x4 => ModemStatus::RxOnGoing,
            0x8 => ModemStatus::HeaderInfoValid,
            _ => ModemStatus::ModemClear,
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverCurrentProtection {
    pub on: bool,
    pub trim: u8 // mA
}
impl From<u8> for OverCurrentProtection {
    fn from(value: u8) -> Self {
        Self {
            on: (value & OCP_ON_MASK) >> 5 == 1,
            trim: value & OCP_TRIM_MASK
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PARamp {
    Ms3_4 = 0x0,
    Ms2 = 0x1,
    Ms1 = 0x2,
    Us500 = 0x3,
    Us250 = 0x4,
    Us125 = 0x5,
    Us100 = 0x6,
    Us62 = 0x7,
    Us50 = 0x8,
    #[default]
    Us40 = 0x9,
    Us31 = 0xa,
    Us25 = 0xb,
    Us20 = 0xc,
    Us15 = 0xd,
    Us12 = 0xe,
    Us10 = 0xf,
}
impl From<u8> for PARamp {
    fn from(value: u8) -> Self {
        match value {
            0x0 => PARamp::Ms3_4,
            0x1 => PARamp::Ms2,
            0x2 => PARamp::Ms1,
            0x3 => PARamp::Us500,
            0x4 => PARamp::Us250,
            0x5 => PARamp::Us125,
            0x6 => PARamp::Us100,
            0x7 => PARamp::Us62,
            0x8 => PARamp::Us50,
            0x9 => PARamp::Us40,
            0xa => PARamp::Us31,
            0xb => PARamp::Us25,
            0xc => PARamp::Us20,
            0xd => PARamp::Us15,
            0xe => PARamp::Us12,
            _ => PARamp::Us10,
        }
    }
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

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use crate::lora::types::{InvertIQ, OverCurrentProtection, FEI};

    #[test]
    fn fei_new_neg_fei_hz_ok() {
        let fei = FEI::new(-2i32, 16f32, 32u32);
        assert_relative_eq!(fei.hz, -0.032, epsilon=1e-3);
    }

    #[test]
    fn fei_new_pos_fei_hz_ok() {
        let fei = FEI::new(8i32, 16f32, 32u32);
        assert_relative_eq!(fei.hz, 0.128, epsilon=1e-3);
    }

    #[test]
    fn fei_new_neg_fei_ppm_ok() {
        let fei = FEI::new(-4i32, 16f32, 32u32);
        assert_relative_eq!(fei.ppm, -2000.0, epsilon=1e-3);
    }

    #[test]
    fn fei_new_pos_fei_ppm_ok() {
        let fei = FEI::new(8i32, 16f32, 32u32);
        assert_relative_eq!(fei.ppm, 4000.0, epsilon=1e-3);
    }

    #[test]
    fn invert_iq_from_u8_ok() {
        let byte = 0b100_0000;
        let invert_iq = InvertIQ::from(byte);
        assert!(invert_iq.rx_path);
        assert!(!invert_iq.tx_path);
    }

    #[test]
    fn over_current_protection_from_u8_ok() {
        let byte = 0b11_0000;
        let ocp = OverCurrentProtection::from(byte);
        assert_eq!(true, ocp.on);
        assert_eq!(0x10, ocp.trim);
    }
}