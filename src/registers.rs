use bitfields::bitfield;
use crate::types::DeviceMode;

pub(crate) trait Register {
    fn addr() -> u8;
}

enum Reg {
    Fifo = 0x00,
    OpMode = 0x01,
    FrMsb = 0x06,
    FrMid = 0x07,
    FrLsb = 0x08,
    PaConfig = 0x09,
    PaRamp = 0x0a,
    Ocp = 0x0b,
    Lna = 0x0c,
    FifoAddrPtr = 0x0d,
    FifoTxBaseAddr = 0x0e,
    FifoRxBaseAddr = 0x0f,
    FifoRxCurrentAddr = 0x10,
    IrqFlagsMask = 0x11,
    IrqFlags = 0x12,
    RxNbBytes = 0x13,
    RxHeaderCntValueMsb = 0x14,
    RxHeaderCntValueLsb = 0x15,
    RxPacketCntValueMsb = 0x16,
    RxPacketCntValueLsb = 0x17,
    ModemStat = 0x18,
    PktSnrValue = 0x19,
    PktRssiValue = 0x1a,
    RssiValue = 0x1b,
    HopChannel = 0x1c,
    ModemConfig1 = 0x1d,
    ModemConfig2 = 0x1e,
    SymbTimeoutLsb = 0x1f,
    PreambleMsb = 0x20,
    PreambleLsb = 0x21,
    PayloadLength = 0x22,
    MaxPayloadLength = 0x23,
    HopPeriod = 0x24,
    FifoRxByteAddr = 0x25,
    ModemConfig3 = 0x26,
    // PpmCorrection ?
    FeiMsb = 0x28,
    FiMid = 0x29,
    FeiLsb = 0x2a,
    RssiWideband = 0x2c,
    // IfFreq2 ?
    // IfFreq1 ?
    DetectOptimize = 0x31,
    InvertIQ1 = 0x33,
    HighBWOptimize1 = 0x36,
    DetectionThreshold = 0x37,
    SyncWord = 0x39,
    HighBWOptimize2 = 0x3a,
    InvertIQ2 = 0x3b,
}

#[bitfield(u8)]
#[derive(Copy, Clone)]
pub(crate) struct RegOpMode {
    long_range_mode: bool,
    access_shared_reg: bool,
    #[bits(2)]
    _pad: u8,
    low_frequency_mode_on: bool,
    #[bits(3)]
    mode: DeviceMode
}
impl Register for RegOpMode {
    fn addr() -> u8 { Reg::OpMode as u8 }
}