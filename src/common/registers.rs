pub const FIFO: u8 = 0x00;

// RegOpMode ---------------------------------------------------------------------------------------
pub const OP_MODE: u8 = 0x01;
pub const OP_MODE_LONG_RANGE_MODE_MASK: u8 = 0x80;
pub const OP_MODE_ACCESS_SHARED_REG_MASK: u8 = 0x40;
pub const OP_MODE_LOW_FREQUENCY_MODE_ON_MASK: u8 = 0x08;
pub const OP_MODE_MODE_MASK: u8 = 0x07;

// -------------------------------------------------------------------------------------------------
pub const FRF_MSB: u8 = 0x06;
pub const FRF_MID: u8 = 0x07;
pub const FRF_LSB: u8 = 0x08;

// RegPaConfig -------------------------------------------------------------------------------------
pub const PA_CONFIG: u8 = 0x09;
pub const PA_CONFIG_PA_SELECT_MASK: u8 = 0x80;
pub const PA_CONFIG_MAX_POWER_MASK: u8 = 0x70;
pub const PA_CONFIG_OUTPUT_POWER_MASK: u8 = 0xf;

// -------------------------------------------------------------------------------------------------
pub const PA_RAMP: u8 = 0x0a;

// RegOcp ------------------------------------------------------------------------------------------
pub const OCP: u8 = 0x0b;
pub const OCP_ON_MASK: u8 = 0x20;
pub const OCP_TRIM_MASK: u8 = 0x1f;

// RegLna ------------------------------------------------------------------------------------------
pub const LNA: u8 = 0x0c;
pub const LNA_GAIN_MASK: u8 = 0xe0;
pub const LNA_BOOST_LF_MASK: u8 = 0x18;
pub const LNA_BOOST_HF_MASK: u8 = 0x3;

// 0x0d..=0x3f modem-specific register page

// RegDioMapping1 ----------------------------------------------------------------------------------
pub const DIO_MAPPING_1: u8 = 0x40;
pub const DIO_MAPPING_1_DIO0_MASK: u8 = 0xc0;
pub const DIO_MAPPING_1_DIO0_SHIFT: u8 = 0x6;
pub const DIO_MAPPING_1_DIO1_MASK: u8 = 0x30;
pub const DIO_MAPPING_1_DIO1_SHIFT: u8 = 0x4;
pub const DIO_MAPPING_1_DIO2_MASK: u8 = 0x0c;
pub const DIO_MAPPING_1_DIO2_SHIFT: u8 = 0x2;
pub const DIO_MAPPING_1_DIO3_MASK: u8 = 0x03;
pub const DIO_MAPPING_1_DIO3_SHIFT: u8 = 0x0;

// RegDioMapping1 ----------------------------------------------------------------------------------
pub const DIO_MAPPING_2: u8 = 0x41;
pub const DIO_MAPPING_2_DIO4_MASK: u8 = 0xc0;
pub const DIO_MAPPING_2_DIO4_SHIFT: u8 = 0x6;
pub const DIO_MAPPING_2_DIO5_MASK: u8 = 0x30;
pub const DIO_MAPPING_2_DIO5_SHIFT: u8 = 0x4;

// -------------------------------------------------------------------------------------------------
pub const VERSION: u8 = 0x42;

// RegPaDac ----------------------------------------------------------------------------------------
pub const PA_DAC: u8 = 0x4d;
pub const PA_DAC_MASK: u8 = 0x07;