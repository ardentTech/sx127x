/// Register mappings for the LoRa modem.
///
/// See: Table 41
pub enum LoraReg {
    Fifo = 0x00,
    // OpMode = 0x01,
    // FrfMsb = 0x06,
    // FrfMid = 0x07,
    // FrfLsb = 0x08,
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
    FeiMsb = 0x28,
    FeiMid = 0x29,
    FeiLsb = 0x2a,
    RssiWideband = 0x2c,
    IfFreq1 = 0x2f,
    IfFreq2 = 0x30,
    DetectOptimize = 0x31,
    InvertIQ = 0x33,
    HighBwOptimize1 = 0x36,
    DetectionThreshold = 0x37,
    SyncWord = 0x39,
    HighBwOptimize2 = 0x3a,
    InvertIQ2 = 0x3b,
    DioMapping1 = 0x40,
    DioMapping2 = 0x41,
    Version = 0x42,
    Tcxo = 0x4b,
    PaDac = 0x4d,
    FormerTemp = 0x5b,
    AgcRef = 0x61,
    AgcThresh1 = 0x62,
    AgcThresh2 = 0x63,
    AgcThresh3 = 0x64,
    Pll = 0x70
}

pub const FIFO: u8 = 0x0;

pub const OP_MODE: u8 = 0x01;
pub const OP_MODE_LONG_RANGE_MODE_MASK: u8 = 0x80;
pub const OP_MODE_ACCESS_SHARED_REG_MASK: u8 = 0x40;
pub const OP_MODE_LOW_FREQUENCY_MODE_ON_MASK: u8 = 0x08;
pub const OP_MODE_MODE_MASK: u8 = 0x07;

pub const FRF_MSB: u8 = 0x06;
pub const FRF_MID: u8 = 0x07;
pub const FRF_LSB: u8 = 0x08;
pub const FIFO_ADDR_PTR: u8 = 0x0d;
pub const FIFO_TX_BASE_ADDR: u8 = 0x0e;
pub const FIFO_RX_BASE_ADDR: u8 = 0x0f;
pub const FIFO_RX_CURRENT_ADDR: u8 = 0x10;
pub const IRQ_FLAGS_MASK: u8 = 0x11;
pub const IRQ_FLAGS: u8 = 0x12;
pub const RX_NB_BYTES: u8 = 0x13;

pub const HOP_CHANNEL: u8 = 0x1c;
pub const HOP_CHANNEL_CRC_ON_PAYLOAD_MASK: u8 = 0x40;

pub const MODEM_CONFIG_1: u8 = 0x1d;
pub const MODEM_CONFIG_1_BW_MASK: u8 = 0xf0;
pub const MODEM_CONFIG_1_CODING_RATE_MASK: u8 = 0xe;
pub const MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK: u8 = 0x1;

pub const MODEM_STAT: u8 = 0x18;

pub const MODEM_CONFIG_2: u8 = 0x1e;
pub const MODEM_CONFIG_2_SPREADING_FACTOR_MASK: u8 = 0xf0;
pub const MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_MASK: u8 = 0x4;

pub const SYMB_TIMEOUT_LSB: u8 = 0x1f;
pub const PREAMBLE_MSB: u8 = 0x20;
pub const PREAMBLE_LSB: u8 = 0x21;
pub const PAYLOAD_LENGTH: u8 = 0x22;

pub const MODEM_CONFIG_3: u8 = 0x26;
pub const MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_FLAG: u8 = 0x8;

// RegImageCal: this is a FSK/OOK reg needed for calibration (hence only pub(crate))
pub(crate) const IMAGE_CAL: u8 = 0x3b;

pub const DIO_MAPPING_1: u8 = 0x40;
pub const DIO_MAPPING_1_DIO0_MASK: u8 = 0xc0;
pub const DIO_MAPPING_1_DIO0_SHIFT: u8 = 0x6;
pub const DIO_MAPPING_1_DIO1_MASK: u8 = 0x30;

pub const DETECT_OPTIMIZE: u8 = 0x31;
pub const DETECT_OPTIMIZE_DETECTION_OPTIMIZE_MASK: u8 = 0x7;

pub const HIGH_BW_OPTIMIZE_1: u8 = 0x36;
pub const DETECTION_THRESHOLD: u8 = 0x37;
pub const HIGH_BW_OPTIMIZE_2: u8 = 0x3a;

pub const DIO_MAPPING_1_DIO1_SHIFT: u8 = 0x4;
pub const VERSION: u8 = 0x42;