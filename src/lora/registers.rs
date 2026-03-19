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

// RegFifo
pub const FIFO: u8 = 0x0;

// RegOpMode
pub const OP_MODE: u8 = 0x01;
pub const OP_MODE_LONG_RANGE_MODE_MASK: u8 = 0x80;
pub const OP_MODE_ACCESS_SHARED_REG_MASK: u8 = 0x40;
pub const OP_MODE_LOW_FREQUENCY_MODE_ON_MASK: u8 = 0x08;
pub const OP_MODE_MODE_MASK: u8 = 0x07;

// RegFrfMsb..Lsb
pub const FRF_MSB: u8 = 0x06;
pub const FRF_MID: u8 = 0x07;
pub const FRF_LSB: u8 = 0x08;

// RegFifoAddrPtr
pub const FIFO_ADDR_PTR: u8 = 0x0d;

// RegFifoTxBaseAddr
pub const FIFO_TX_BASE_ADDR: u8 = 0x0e;

// RegFifoRxBaseAddr
pub const FIFO_RX_BASE_ADDR: u8 = 0x0f;

pub const FIFO_RX_CURRENT_ADDR: u8 = 0x10;

// RegIrqFlags
pub const IRQ_FLAGS: u8 = 0x12;

pub const RX_NB_BYTES: u8 = 0x13;

// RegHopChannel
pub const HOP_CHANNEL: u8 = 0x1c;
pub const HOP_CHANNEL_CRC_ON_PAYLOAD_MASK: u8 = 0x40;

// RegModemConfig2
pub const MODEM_CONFIG_2: u8 = 0x1e;

// RegSymbTimeoutLsb
pub const SYMB_TIMEOUT_LSB: u8 = 0x1f;

// RegPayloadLength
pub const PAYLOAD_LENGTH: u8 = 0x22;

// RegImageCal: this is a FSK/OOK reg needed for calibration (hence only pub(crate))
// TODO find a way to avoid duplicating this...
pub(crate) const IMAGE_CAL: u8 = 0x3b;

// RegDioMapping1
pub const DIO_MAPPING_1: u8 = 0x40;
pub const DIO_MAPPING_1_DIO0_MASK: u8 = 0xc0;
pub const DIO_MAPPING_1_DIO0_SHIFT: u8 = 0x6;
pub const DIO_MAPPING_1_DIO1_MASK: u8 = 0x30;
pub const DIO_MAPPING_1_DIO1_SHIFT: u8 = 0x4;