pub const FIFO: u8 = 0x0;

pub const OP_MODE: u8 = 0x01;
pub const OP_MODE_LONG_RANGE_MODE_MASK: u8 = 0x80;
pub const OP_MODE_ACCESS_SHARED_REG_MASK: u8 = 0x40;
pub const OP_MODE_LOW_FREQUENCY_MODE_ON_MASK: u8 = 0x08;
pub const OP_MODE_MODE_MASK: u8 = 0x07;

pub const FRF_MSB: u8 = 0x06;
pub const FRF_MID: u8 = 0x07;
pub const FRF_LSB: u8 = 0x08;
pub const PA_RAMP: u8 = 0x0a;
pub const PA_RAMP_MASK: u8 = 0xf;

pub const OCP: u8 = 0x0b;
pub const OCP_ON_MASK: u8 = 0x20;
pub const OCP_TRIM_MASK: u8 = 0x1f;

// begin lora page registers (0x0d) ----------------------------------------------------------------
pub const FIFO_ADDR_PTR: u8 = 0x0d;
pub const FIFO_TX_BASE_ADDR: u8 = 0x0e;
pub const FIFO_RX_BASE_ADDR: u8 = 0x0f;
pub const FIFO_RX_CURRENT_ADDR: u8 = 0x10;
pub const IRQ_FLAGS_MASK: u8 = 0x11;
pub const IRQ_FLAGS: u8 = 0x12;
pub const RX_NB_BYTES: u8 = 0x13;
pub const RX_HEADER_CNT_VALUE_MSB: u8 = 0x14;
pub const RX_HEADER_CNT_VALUE_LSB: u8 = 0x15;
pub const RX_PACKET_CNT_VALUE_MSB: u8 = 0x16;
pub const RX_PACKET_CNT_VALUE_LSB: u8 = 0x17;

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
pub const FEI_MSB: u8 = 0x28;
pub const FEI_MID: u8 = 0x29;
pub const FEI_LSB: u8 = 0x2a;
pub const MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_FLAG: u8 = 0x8;

// RegImageCal: this is a FSK/OOK reg needed for calibration (hence only pub(crate))
pub(crate) const IMAGE_CAL: u8 = 0x3b;

// end lora page registers (0x3f) ------------------------------------------------------------------

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