use bitfields::bitfield;
use crate::lora::types::*;
// TODO mark RO bitfields

pub(crate) trait Addressable {
    fn addr() -> u8;
}

pub(crate) enum Reg {
    Fifo = 0x00,
    OpMode = 0x01,
    // 0x02..=0x05: reserved
    FrMsb = 0x06,
    FrMid = 0x07,
    FrLsb = 0x08,
    PaConfig = 0x09,
    PaRamp = 0x0a, // TODO
    Ocp = 0x0b,
    Lna = 0x0c,
    // TODO 0d..=3f require AccessSharedReg on RegOpMode to be set (see 4.4)
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
    //
    DioMapping1 = 0x40,
    DioMapping2 = 0x41,
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegDetectOptimize {
    automatic_if_on: bool,
    #[bits(4)]
    _pad: u8,
    #[bits(3)]
    detection_optimize: u8
}
impl Addressable for RegDetectOptimize {
    fn addr() -> u8 { Reg::DetectOptimize as u8 }
}
impl RegDetectOptimize {
    // TODO not happy with this name
    pub(crate) fn update(&mut self, spreading_factor: SpreadingFactor) {
        match spreading_factor {
            SpreadingFactor::Sf6  => self.set_detection_optimize(0x05),
            _ => self.set_detection_optimize(0x03),
        }
    }
}

#[bitfield(u8)]
#[derive(Copy, Clone)]
pub(crate) struct RegDetectionThreshold {
    detection_threshold: u8
}
impl Addressable for RegDetectionThreshold {
    fn addr() -> u8 { Reg::DetectionThreshold as u8 }
}
impl RegDetectionThreshold {
    // TODO not happy with this name
    pub(crate) fn update(&mut self, spreading_factor: SpreadingFactor) {
        match spreading_factor {
            SpreadingFactor::Sf6 => self.set_detection_threshold(0x0c),
            _ => self.set_detection_threshold(0x0a),
        }
    }
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegDioMapping1 { // I think this is the same between the two modems
    #[bits(2)]
    dio0: u8,
    #[bits(2)]
    dio1: u8,
    #[bits(2)]
    dio2: u8,
    #[bits(2)]
    dio3: u8,
}
impl Addressable for RegDioMapping1 {
    fn addr() -> u8 { Reg::DioMapping1 as u8 }
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegDioMapping2 { // I think this is the same between the two modems
    #[bits(2)]
    dio4: u8,
    #[bits(2)]
    dio5: u8,
    #[bits(3)]
    _pad: u8,
    map_preamble_detect: bool
}
impl Addressable for RegDioMapping2 {
    fn addr() -> u8 { Reg::DioMapping2 as u8 }
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegHopChannel {
    pll_timeout: bool,
    crc_on_payload: bool,
    #[bits(6)]
    _pad: u8,
}
impl Addressable for RegHopChannel {
    fn addr() -> u8 { Reg::HopChannel as u8 }
}


#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegIrqFlagsMask {
    rx_timeout_mask: bool,
    rx_done_mask: bool,
    payload_crc_error_mask: bool,
    valid_header_mask: bool,
    tx_done_mask: bool,
    cad_done_mask: bool,
    fhss_change_channel_mask: bool,
    cad_detected_mask: bool,
}
impl Addressable for RegIrqFlagsMask {
    fn addr() -> u8 { Reg::IrqFlagsMask as u8 }
}
impl RegIrqFlagsMask {
    pub(crate) fn mask(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::CadDetected => self.set_cad_detected_mask(true),
            Interrupt::FhssChangeChannel => self.set_fhss_change_channel_mask(true),
            Interrupt::CadDone => self.set_cad_done_mask(true),
            Interrupt::TxDone => self.set_tx_done_mask(true),
            Interrupt::ValidHeader => self.set_valid_header_mask(true),
            Interrupt::PayloadCrcError => self.set_payload_crc_error_mask(true),
            Interrupt::RxDone => self.set_rx_done_mask(true),
            Interrupt::RxTimeout => self.set_rx_timeout_mask(true),
        }
    }
    pub(crate) fn unmask(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::CadDetected => self.set_cad_detected_mask(false),
            Interrupt::FhssChangeChannel => self.set_fhss_change_channel_mask(false),
            Interrupt::CadDone => self.set_cad_done_mask(false),
            Interrupt::TxDone => self.set_tx_done_mask(false),
            Interrupt::ValidHeader => self.set_valid_header_mask(false),
            Interrupt::PayloadCrcError => self.set_payload_crc_error_mask(false),
            Interrupt::RxDone => self.set_rx_done_mask(false),
            Interrupt::RxTimeout => self.set_rx_timeout_mask(false),
        }
    }
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegIrqFlags {
    rx_timeout: bool,
    rx_done: bool,
    payload_crc_error: bool,
    valid_header: bool,
    tx_done: bool,
    cad_done: bool,
    fhss_change_channel: bool,
    cad_detected: bool,
}
impl Addressable for RegIrqFlags {
    fn addr() -> u8 { Reg::IrqFlags as u8 }
}
impl RegIrqFlags {
    pub(crate) fn clear_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::CadDetected => self.set_cad_detected(true),
            Interrupt::FhssChangeChannel => self.set_fhss_change_channel(true),
            Interrupt::CadDone => self.set_cad_done(true),
            Interrupt::TxDone => self.set_tx_done(true),
            Interrupt::ValidHeader => self.set_valid_header(true),
            Interrupt::PayloadCrcError => self.set_payload_crc_error(true),
            Interrupt::RxDone => self.set_rx_done(true),
            Interrupt::RxTimeout => self.set_rx_timeout(true),
        }
    }

    pub(crate) fn interrupt_triggered(&self, interrupt: Interrupt) -> bool {
        match interrupt {
            Interrupt::CadDetected => self.cad_detected(),
            Interrupt::FhssChangeChannel => self.fhss_change_channel(),
            Interrupt::CadDone => self.cad_done(),
            Interrupt::TxDone => self.tx_done(),
            Interrupt::ValidHeader => self.valid_header(),
            Interrupt::PayloadCrcError => self.payload_crc_error(),
            Interrupt::RxDone => self.rx_done(),
            Interrupt::RxTimeout => self.rx_timeout(),
        }
    }

    // Determines if a RX packet terminated successfully.
    pub(crate) fn packet_rx_termination_ok(&self, crc_on_payload: bool) -> bool {
        let bits = self.into_bits() >> 4;
        if crc_on_payload {
            bits & 0xf == 0
        } else {
            bits & 0xc == 0 && bits & 0x1 == 0
        }
    }
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegModemConfig1 {
    #[bits(4)]
    bandwidth: Bandwidth,
    #[bits(3)]
    coding_rate: CyclicErrorCoding,
    implicit_header_mode_on: bool
}
impl Addressable for RegModemConfig1 {
    fn addr() -> u8 { Reg::ModemConfig1 as u8 }
}
#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegModemConfig2 {
    #[bits(4)]
    spreading_factor: SpreadingFactor,
    tx_continuous_mode: bool,
    rx_payload_crc_on: bool,
    #[bits(2)]
    symbol_timeout_msb: u8
}
impl Addressable for RegModemConfig2 {
    fn addr() -> u8 { Reg::ModemConfig2 as u8 }
}

#[bitfield(u8, order = msb)]
#[derive(Copy, Clone)]
pub(crate) struct RegModemStat {
    #[bits(3)]
    rx_coding_rate: u8,
    modem_clear: bool,
    header_info_valid: bool,
    rx_on_going: bool,
    signal_synchronized: bool,
    signal_detected: bool
}
impl Addressable for RegModemStat {
    fn addr() -> u8 { Reg::ModemStat as u8 }
}
impl Into<RxStatus> for RegModemStat {
    fn into(self) -> RxStatus {
        if self.modem_clear() {
            RxStatus::ModemClear
        } else if self.header_info_valid() {
            RxStatus::HeaderInfoValid
        } else if self.rx_on_going() {
            RxStatus::RxOnGoing
        } else if self.signal_synchronized() {
            RxStatus::SignalSynchronized
        } else if self.signal_detected() {
            RxStatus::SignalDetected
        } else {
            RxStatus::Unknown
        }
    }
}

#[bitfield(u8, order = msb)]
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
impl Addressable for RegOpMode {
    fn addr() -> u8 { Reg::OpMode as u8 }
}

#[bitfield(u8)]
#[derive(Copy, Clone)]
pub(crate) struct RegPaConfig {
    pa_select: bool,
    #[bits(3)]
    max_power: u8,
    #[bits(4)]
    output_power: u8
}
impl Addressable for RegPaConfig {
    fn addr() -> u8 { Reg::PaConfig as u8 }
}

#[bitfield(u8)]
#[derive(Copy, Clone)]
pub(crate) struct RegSymbTimeoutLsb {
    symb_timeout: u8
}
impl Addressable for RegSymbTimeoutLsb {
    fn addr() -> u8 { Reg::SymbTimeoutLsb as u8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_dio0() {
        let mut byte = RegDioMapping1::from_bits(0b0);
        byte.set_dio0(Dio0::TxDone as u8);
        assert_eq!(byte.dio0(), Dio0::TxDone as u8);
    }

    #[test]
    fn test_clear_interrupt_cad_detected() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::CadDetected);
        assert_eq!(0b1, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_fhss_change_channel() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::FhssChangeChannel);
        assert_eq!(0b10, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_cad_done() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::CadDone);
        assert_eq!(0b100, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_tx_done() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::TxDone);
        assert_eq!(0b1000, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_valid_header() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::ValidHeader);
        assert_eq!(0b1_0000, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_payload_crc_error() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::PayloadCrcError);
        assert_eq!(0b10_0000, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_rx_done() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::RxDone);
        assert_eq!(0b100_0000, byte.into_bits());
    }

    #[test]
    fn test_clear_interrupt_rx_timeout() {
        let mut byte = RegIrqFlags::from_bits(0b0);
        byte.clear_interrupt(Interrupt::RxTimeout);
        assert_eq!(0b1000_0000, byte.into_bits());
    }

    #[test]
    fn test_interrupt_triggered_cad_detected_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::CadDetected));
    }

    #[test]
    fn test_interrupt_triggered_cad_detected_true() {
        let byte = RegIrqFlags::from_bits(0b1);
        assert!(byte.interrupt_triggered(Interrupt::CadDetected));
    }

    #[test]
    fn test_interrupt_triggered_fhss_change_channel_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::FhssChangeChannel));
    }

    #[test]
    fn test_interrupt_triggered_fhss_change_channel_true() {
        let byte = RegIrqFlags::from_bits(0b10);
        assert!(byte.interrupt_triggered(Interrupt::FhssChangeChannel));
    }

    #[test]
    fn test_interrupt_triggered_cad_done_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::CadDone));
    }

    #[test]
    fn test_interrupt_triggered_cad_done_true() {
        let byte = RegIrqFlags::from_bits(0b100);
        assert!(byte.interrupt_triggered(Interrupt::CadDone));
    }

    #[test]
    fn test_interrupt_triggered_tx_done_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::TxDone));
    }

    #[test]
    fn test_interrupt_triggered_tx_done_true() {
        let byte = RegIrqFlags::from_bits(0b1000);
        assert!(byte.interrupt_triggered(Interrupt::TxDone));
    }

    #[test]
    fn test_interrupt_triggered_valid_header_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::ValidHeader));
    }

    #[test]
    fn test_interrupt_triggered_valid_header_true() {
        let byte = RegIrqFlags::from_bits(0b1_0000);
        assert!(byte.interrupt_triggered(Interrupt::ValidHeader));
    }

    #[test]
    fn test_interrupt_triggered_payload_crc_error_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::PayloadCrcError));
    }

    #[test]
    fn test_interrupt_triggered_payload_crc_error_true() {
        let byte = RegIrqFlags::from_bits(0b10_0000);
        assert!(byte.interrupt_triggered(Interrupt::PayloadCrcError));
    }

    #[test]
    fn test_interrupt_triggered_rx_done_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::RxDone));
    }

    #[test]
    fn test_interrupt_triggered_rx_done_true() {
        let byte = RegIrqFlags::from_bits(0b100_0000);
        assert!(byte.interrupt_triggered(Interrupt::RxDone));
    }

    #[test]
    fn test_interrupt_triggered_rx_timeout_false() {
        let byte = RegIrqFlags::from_bits(0b0);
        assert!(!byte.interrupt_triggered(Interrupt::RxTimeout));
    }

    #[test]
    fn test_interrupt_triggered_rx_timeout_true() {
        let byte = RegIrqFlags::from_bits(0b1000_0000);
        assert!(byte.interrupt_triggered(Interrupt::RxTimeout));
    }

    #[test]
    fn test_packet_rx_termination_ok_true_crc_on_payload_true() {
        let byte = RegIrqFlags::from_bits(0x0f);
        assert!(byte.packet_rx_termination_ok(true))
    }

    #[test]
    fn test_packet_rx_termination_ok_false_crc_on_payload_true() {
        let byte = RegIrqFlags::from_bits(0xff);
        assert!(!byte.packet_rx_termination_ok(true))
    }

    #[test]
    fn test_packet_rx_termination_ok_false_crc_on_payload_false() {
        let byte = RegIrqFlags::from_bits(0xff);
        assert!(!byte.packet_rx_termination_ok(false))
    }

    #[test]
    fn test_packet_rx_termination_ok_true_crc_on_payload_false() {
        let byte = RegIrqFlags::from_bits(0xdd);
        assert!(!byte.packet_rx_termination_ok(false))
    }
}