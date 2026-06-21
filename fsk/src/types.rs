use sx127x_common::bits::get_bits;
use sx127x_common::error::Sx127xError;
use crate::registers::*;
use crate::types::FromReceive::*;
use crate::validate;

pub enum AgcThreshold {
    Step1(u8),
    Step2(u8),
    Step3(u8),
    Step4(u8),
    Step5(u8),
}
impl AgcThreshold {
    pub(crate) fn mask(&self) -> u8 {
        match self {
            AgcThreshold::Step1(_) => AGC_THRESH_1_AGC_STEP_1_MASK,
            AgcThreshold::Step2(_) => AGC_THRESH_2_AGC_STEP_2_MASK,
            AgcThreshold::Step3(_) => AGC_THRESH_2_AGC_STEP_3_MASK,
            AgcThreshold::Step4(_) => AGC_THRESH_3_AGC_STEP_4_MASK,
            AgcThreshold::Step5(_) => AGC_THRESH_3_AGC_STEP_5_MASK,
        }
    }

    pub(crate) fn offset(&self) -> u8 {
        match self {
            AgcThreshold::Step1(_) => AGC_THRESH_1_AGC_STEP_1_OFFSET,
            AgcThreshold::Step2(_) => AGC_THRESH_2_AGC_STEP_2_OFFSET,
            AgcThreshold::Step3(_) => AGC_THRESH_2_AGC_STEP_3_OFFSET,
            AgcThreshold::Step4(_) => AGC_THRESH_3_AGC_STEP_4_OFFSET,
            AgcThreshold::Step5(_) => AGC_THRESH_3_AGC_STEP_5_OFFSET,
        }
    }

    pub(crate) fn reg_addr(&self) -> u8 {
        match self {
            AgcThreshold::Step1(_) => AGC_THRESH_1,
            AgcThreshold::Step2(_) | AgcThreshold::Step3(_) => AGC_THRESH_2,
            AgcThreshold::Step4(_) | AgcThreshold::Step5(_) => AGC_THRESH_3,
        }
    }

    pub(crate) fn value(&self) -> u8 {
        match self {
            AgcThreshold::Step1(a) => *a,
            AgcThreshold::Step2(b) => *b,
            AgcThreshold::Step3(c) => *c,
            AgcThreshold::Step4(d) => *d,
            AgcThreshold::Step5(e) => *e,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AutoRestartRxMode {
    Off = 0x0,
    On = 0x1,
    #[default]
    OnWaitForPllLock = 0x2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bandwidth {
    Bw2_6kHz,
    Bw3_1kHz,
    Bw3_9kHz,
    Bw5_2kHz,
    Bw6_3kHz,
    Bw7_8kHz,
    Bw10_4kHz,
    Bw12_5kHz,
    Bw15_6kHz,
    Bw20_8kHz,
    Bw25kHz,
    Bw31_3kHz,
    Bw41_7kHz,
    Bw50kHz,
    Bw62_5kHz,
    Bw83_3kHz,
    Bw100kHz,
    Bw125kHz,
    Bw166_7kHz,
    Bw200kHz,
    Bw250kHz,
}

pub(crate) struct BwConfig {
    pub(crate) exp: u8,
    pub(crate) mant: u8,
}
impl BwConfig {
    fn new(exp: u8, mant: u8) -> Self { Self { exp, mant } }
}

impl From<Bandwidth> for BwConfig {
    /// See: datasheet table 40
    fn from(value: Bandwidth) -> Self {
        match value {
            Bandwidth::Bw2_6kHz => BwConfig::new(7, 2),
            Bandwidth::Bw3_1kHz => BwConfig::new(7, 1),
            Bandwidth::Bw3_9kHz => BwConfig::new(7, 0),
            Bandwidth::Bw5_2kHz => BwConfig::new(6, 2),
            Bandwidth::Bw6_3kHz => BwConfig::new(6, 1),
            Bandwidth::Bw7_8kHz => BwConfig::new(6, 0),
            Bandwidth::Bw10_4kHz => BwConfig::new(5, 2),
            Bandwidth::Bw12_5kHz => BwConfig::new(5, 1),
            Bandwidth::Bw15_6kHz => BwConfig::new(5, 0),
            Bandwidth::Bw20_8kHz => BwConfig::new(4, 2),
            Bandwidth::Bw25kHz => BwConfig::new(4, 1),
            Bandwidth::Bw31_3kHz => BwConfig::new(4, 0),
            Bandwidth::Bw41_7kHz => BwConfig::new(3, 2),
            Bandwidth::Bw50kHz => BwConfig::new(3, 1),
            Bandwidth::Bw62_5kHz => BwConfig::new(3, 0),
            Bandwidth::Bw83_3kHz => BwConfig::new(2, 2),
            Bandwidth::Bw100kHz => BwConfig::new(2, 1),
            Bandwidth::Bw125kHz => BwConfig::new(2, 0),
            Bandwidth::Bw166_7kHz => BwConfig::new(1, 2),
            Bandwidth::Bw200kHz => BwConfig::new(1, 1),
            Bandwidth::Bw250kHz => BwConfig::new(1, 0),
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ClkOut {
    Fxosc = 0x0,
    FxoscDiv2 = 0x1,
    FxoscDiv4 = 0x2,
    FxoscDiv8 = 0x3,
    FxoscDiv16 = 0x4,
    FxoscDiv32 = 0x5,
    RC = 0x6,
    #[default]
    Off = 0x7
}

// -------------------------------------------------------------------------------------------------
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
            0x0 => DeviceMode::SLEEP,
            0x1 => DeviceMode::STDBY,
            0x2 => DeviceMode::FSTX,
            0x3 => DeviceMode::TX,
            0x4 => DeviceMode::FSRX,
            _ => DeviceMode::RX,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FastFrequencyHoppingMode {
    FstxOrFsrxRequested = 0x0,
    RegFrfLsbWritten = 0x1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModulationType {
    FSK = 0x00,
    OOK = 0x01
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OokPeakThreshDec {
    #[default]
    OncePerChip = 0x0,
    OnceEveryTwoChips = 0x1,
    OnceEveryFourChips = 0x2,
    OnceEveryEightChips = 0x3,
    TwiceInEachChip = 0x4,
    FourTimesInEachChip = 0x5,
    EightTimesInEachChip = 0x6,
    SixteenTimesInEachChips = 0x7,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OokAverageOffset {
    #[default]
    Offset0dB = 0x0,
    Offset2dB = 0x1,
    Offset4dB = 0x2,
    Offset6dB = 0x3,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OokAverageThreshFilt {
    Divisor32xPi = 0x0,
    Divisor8xPi = 0x1,
    #[default]
    Divisor4xPi = 0x2,
    Divisor2xPi = 0x3,
}
pub struct OokAvg {
    pub ook_peak_thresh_dec: OokPeakThreshDec,
    pub ook_average_offset: OokAverageOffset,
    pub ook_average_thresh_filt: OokAverageThreshFilt
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OokThreshType {
    FixedThreshold = 0x0,
    #[default]
    PeakMode = 0x1,
    AverageMode = 0x2
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OokPeakThreshStep {
    #[default]
    Step0_5dB,
    Step1_0dB,
    Step1_5dB,
    Step2_0dB,
    Step3_0dB,
    Step4_0dB,
    Step5_0dB,
    Step6_0dB,
}

pub struct OokPeakConfig {
    pub bit_sync_on: bool,
    pub ook_thresh_type: OokThreshType,
    pub ook_peak_thresh: OokPeakThreshStep
}
impl Default for OokPeakConfig {
    fn default() -> Self {
        Self {
            bit_sync_on: true,
            ook_thresh_type: OokThreshType::default(),
            ook_peak_thresh: OokPeakThreshStep::default()
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AddressFiltering {
    #[default]
    None = 0x0,
    MatchNodeAddress = 0x1,
    MatchNodeOrBroadcastAddress = 0x2,
}

impl From<u8> for AddressFiltering {
    fn from(value: u8) -> Self {
        match value {
            0x1 => AddressFiltering::MatchNodeAddress,
            0x2 => AddressFiltering::MatchNodeOrBroadcastAddress,
            _ => AddressFiltering::None
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CrcWhiteningType {
    #[default]
    CcittStandard = 0x0,
    IbmAlternate = 0x1,
}

impl From<u8> for CrcWhiteningType {
    fn from(value: u8) -> Self {
        match value {
            0x1 => CrcWhiteningType::IbmAlternate,
            _ => CrcWhiteningType::CcittStandard
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DcFree {
    #[default]
    None = 0x0,
    Manchester = 0x1,
    Whitening = 0x2,
}

impl From<u8> for DcFree {
    fn from(value: u8) -> Self {
        match value {
            0x1 => DcFree::Manchester,
            0x2 => DcFree::Whitening,
            _ => DcFree::None,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TxStartCondition {
    FifoBytesExceedsThreshold = 0x0,
    #[default]
    FifoNotEmpty = 0x1,
}
impl From<u8> for TxStartCondition {
    fn from(value: u8) -> Self {
        match value {
            0x0 => TxStartCondition::FifoBytesExceedsThreshold,
            _ => TxStartCondition::FifoNotEmpty,
        }
    }
}

pub struct FifoThreshold(pub(crate) u8);
impl FifoThreshold {
    pub fn new(value: u8) -> Result<Self, Sx127xError<()>> {
        if !validate::fifo_threshold(value) { Err(Sx127xError::InvalidInput) } else { Ok(Self(value)) }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LowBatteryThreshold {
    Trim1_695V = 0x0,
    Trim1_764V = 0x1,
    #[default]
    Trim1_835V = 0x2,
    Trim1_905V = 0x3,
    Trim1_976V = 0x4,
    Trim2_045V = 0x5,
    Trim2_116V = 0x6,
    Trim2_185V = 0x7,
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketFormat {
    FixedLength = 0x0,
    #[default]
    VariableLength = 0x1,
    // TODO unlimited? see p74
}
impl From<u8> for PacketFormat {
    fn from(value: u8) -> Self {
        match value {
            0x0 => PacketFormat::FixedLength,
            _ => PacketFormat::VariableLength
        }
    }
}

pub struct PacketConfig {
    pub address_filtering: AddressFiltering,
    pub beacon_on: bool,
    pub crc_auto_clear_off: bool,
    pub crc_on: bool,
    pub crc_whitening_type: CrcWhiteningType,
    pub dc_free: DcFree,
    pub io_home_on: bool,
    pub packet_format: PacketFormat,
    pub payload_length: u16,
}

impl Default for PacketConfig {
    fn default() -> Self {
        Self {
            address_filtering: AddressFiltering::default(),
            beacon_on: false,
            crc_auto_clear_off: false,
            crc_on: true,
            crc_whitening_type: CrcWhiteningType::default(),
            dc_free: DcFree::default(),
            io_home_on: false,
            packet_format: PacketFormat::default(),
            payload_length: 0x40,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PreambleDetectorSize {
    OneByte = 0x0,
    #[default]
    TwoBytes = 0x1,
    ThreeBytes = 0x2,
}
impl From<u8> for PreambleDetectorSize {
    fn from(value: u8) -> Self {
        match value {
            0x0 => PreambleDetectorSize::OneByte,
            0x2 => PreambleDetectorSize::ThreeBytes,
            _ => PreambleDetectorSize::TwoBytes,
        }
    }
}

// TODO verify this with example
// TODO "Threshold" or is tolerance more appropriate?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreambleDetectorThreshold(pub(crate) u8);
impl PreambleDetectorThreshold {
    pub fn new(value: u8) -> Result<Self, Sx127xError<()>> {
        if !validate::preamble_detector_threshold(value) { Err(Sx127xError::InvalidInput) } else { Ok(Self(value)) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreambleDetector {
    pub on: bool,
    pub size: PreambleDetectorSize,
    pub tolerance: PreambleDetectorThreshold
}

impl Default for PreambleDetector {
    fn default() -> Self {
        Self {
            on: true,
            size: PreambleDetectorSize::default(),
            tolerance: PreambleDetectorThreshold(0x0a)
        }
    }
}
impl From<u8> for PreambleDetector {
    fn from(value: u8) -> Self {
        Self {
            on: get_bits(value, PREAMBLE_DETECT_PREAMBLE_DETECTOR_ON_MASK, PREAMBLE_DETECT_PREAMBLE_DETECTOR_ON_OFFSET) == 1,
            size: PreambleDetectorSize::from(get_bits(value, PREAMBLE_DETECT_PREAMBLE_DETECTOR_SIZE_MASK, PREAMBLE_DETECT_PREAMBLE_DETECTOR_SIZE_OFFSET)),
            tolerance: PreambleDetectorThreshold(get_bits(value, PREAMBLE_DETECT_PREAMBLE_DETECTOR_TOL_MASK, PREAMBLE_DETECT_PREAMBLE_DETECTOR_TOL_OFFSET))
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RxConfig {
    pub afc_auto_on: bool,
    pub agc_auto_on: bool,
    pub restart_rx_on_collision: bool,
    pub rx_trigger: u8
}

impl Default for RxConfig {
    fn default() -> Self {
        Self {
            afc_auto_on: false,
            agc_auto_on: true,
            restart_rx_on_collision: false,
            rx_trigger: 0x06
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RssiSmoothing {
    Samples2 = 0x0,
    Samples4 = 0x1,
    #[default]
    Samples8 = 0x2,
    Samples16 = 0x3,
    Samples32 = 0x4,
    Samples64 = 0x5,
    Samples128 = 0x6,
    Samples256 = 0x7,
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum IdleMode {
    #[default]
    Standby = 0x0,
    Sleep = 0x1,
}
impl From<u8> for IdleMode {
    fn from(value: u8) -> Self {
        match value {
            0x1 => IdleMode::Sleep,
            _ => IdleMode::Standby,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FromStart {
    #[default]
    LowPowerSelection = 0x0,
    ReceiveState = 0x1,
    TransmitState = 0x2,
    TransmitStateOnFifoLevelInterrupt = 0x3,
}
impl From<u8> for FromStart {
    fn from(value: u8) -> Self {
        match value {
            0x1 => FromStart::ReceiveState,
            0x2 => FromStart::TransmitState,
            0x3 => FromStart::TransmitStateOnFifoLevelInterrupt,
            _ => FromStart::LowPowerSelection,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LowPowerSelection {
    #[default]
    SequencerOffState = 0x0,
    IdleState = 0x1,
}
impl From<u8> for LowPowerSelection {
    fn from(value: u8) -> Self {
        match value {
            0x1 => LowPowerSelection::IdleState,
            _=> LowPowerSelection::SequencerOffState,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FromIdle {
    #[default]
    TransmitState = 0x0,
    ReceiveState = 0x1,
}
impl From<u8> for FromIdle {
    fn from(value: u8) -> Self {
        match value {
            0x1 => FromIdle::ReceiveState,
            _ => FromIdle::TransmitState,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FromTransmit {
    #[default]
    LowPowerSelection = 0x0,
    ReceiveState = 0x1,
}
impl From<u8> for FromTransmit {
    fn from(value: u8) -> Self {
        match value {
            0x1 => FromTransmit::ReceiveState,
            _ => FromTransmit::LowPowerSelection,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FromReceive {
    #[default]
    Disabled = 0x0,
    PacketReceivedStateOnPayloadReadyInterrupt = 0x1,
    LowPowerSelectionOnPayloadReadyInterrupt = 0x2,
    PacketReceivedStateOnCrcOkInterrupt = 0x3,
    SequencerOffStateOnRssiInterrupt = 0x4,
    SequencerOffStateOnSyncAddressInterrupt = 0x5,
    SequencerOffStateOnPreambleDetectInterrupt = 0x6,
}
impl From<u8> for FromReceive {
    fn from(value: u8) -> Self {
        match value {
            0x1 => PacketReceivedStateOnPayloadReadyInterrupt,
            0x2 => LowPowerSelectionOnPayloadReadyInterrupt,
            0x3 => PacketReceivedStateOnCrcOkInterrupt,
            0x4 => SequencerOffStateOnRssiInterrupt,
            0x5 => SequencerOffStateOnSyncAddressInterrupt,
            0x6 => SequencerOffStateOnPreambleDetectInterrupt,
            _ => Disabled
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FromRxTimeout {
    #[default]
    ReceiveState = 0x0,
    TransmitState = 0x1,
    LowPowerSelection = 0x2,
    SequencerOffState = 0x3,
}
impl From<u8> for FromRxTimeout {
    fn from(value: u8) -> Self {
        match value {
            0x1 => FromRxTimeout::TransmitState,
            0x2 => FromRxTimeout::LowPowerSelection,
            0x3 => FromRxTimeout::SequencerOffState,
            _ => FromRxTimeout::ReceiveState,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FromPacketReceived {
    #[default]
    SequencerOffState = 0x0,
    TransmitStateOnFifoEmptyInterrupt = 0x1,
    LowPowerSelection = 0x2,
    ReceiveViaFsMode = 0x3,
    ReceiveState = 0x4,
}
impl From<u8> for FromPacketReceived {
    fn from(value: u8) -> Self {
        match value {
            0x1 => FromPacketReceived::TransmitStateOnFifoEmptyInterrupt,
            0x2 => FromPacketReceived::LowPowerSelection,
            0x3 => FromPacketReceived::ReceiveViaFsMode,
            0x4 => FromPacketReceived::ReceiveState,
            _ => FromPacketReceived::SequencerOffState,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SequencerTransitions {
    pub from_idle: FromIdle,
    pub from_packet_received: FromPacketReceived,
    pub from_receive: FromReceive,
    pub from_rx_timeout: FromRxTimeout,
    pub from_start: FromStart,
    pub from_transmit: FromTransmit,
    pub idle_mode: IdleMode,
    pub low_power_selection: LowPowerSelection,
}
impl SequencerTransitions {
    pub(crate) fn set_config1(&mut self, byte: u8) {
        self.idle_mode = IdleMode::from(get_bits(byte, SEQ_CONFIG_1_IDLE_MODE_MASK, SEQ_CONFIG_1_IDLE_MODE_OFFSET));
        self.from_start = FromStart::from(get_bits(byte, SEQ_CONFIG_1_FROM_START_MASK, SEQ_CONFIG_1_FROM_START_OFFSET));
        self.low_power_selection = LowPowerSelection::from(get_bits(byte, SEQ_CONFIG_1_LOW_POWER_SELECTION_MASK, SEQ_CONFIG_1_LOW_POWER_SELECTION_OFFSET));
        self.from_idle = FromIdle::from(get_bits(byte, SEQ_CONFIG_1_FROM_IDLE_MASK, SEQ_CONFIG_1_FROM_IDLE_OFFSET));
        self.from_transmit = FromTransmit::from(get_bits(byte, SEQ_CONFIG_1_FROM_TRANSMIT_MASK, SEQ_CONFIG_1_FROM_TRANSMIT_OFFSET));
    }

    pub(crate) fn set_config2(&mut self, byte: u8) {
        self.from_receive = FromReceive::from(get_bits(byte, SEQ_CONFIG_2_FROM_RECEIVE_MASK, SEQ_CONFIG_2_FROM_RECEIVE_OFFSET));
        self.from_rx_timeout = FromRxTimeout::from(get_bits(byte, SEQ_CONFIG_2_FROM_RX_TIMEOUT_MASK, SEQ_CONFIG_2_FROM_RX_TIMEOUT_OFFSET));
        self.from_packet_received = FromPacketReceived::from(get_bits(byte, SEQ_CONFIG_2_FROM_PACKET_RECEIVED_MASK, SEQ_CONFIG_2_FROM_PACKET_RECEIVED_OFFSET));
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyncConfig {
    pub auto_restart_rx_mode: AutoRestartRxMode,
    pub preamble_polarity: bool,
    pub sync_on: bool,
    pub sync_size: u8
}
impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_restart_rx_mode: Default::default(),
            preamble_polarity: false,
            sync_on: true,
            sync_size: 0x3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TempThreshold {
    Threshold5C = 0x0,
    #[default]
    Threshold10C = 0x1,
    Threshold15C = 0x2,
    Threshold20C = 0x3,
}
impl From<u8> for TempThreshold {
    fn from(value: u8) -> Self {
        match value {
            0x0 => TempThreshold::Threshold5C,
            0x1 => TempThreshold::Threshold10C,
            0x2 => TempThreshold::Threshold15C,
            _ => TempThreshold::Threshold20C,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TimerConfig {
    #[default]
    Disabled = 0x0,
    Resolution64us = 0x1,
    Resolution4_1ms = 0x2,
    Resolution262ms = 0x3,
}