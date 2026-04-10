pub const FIFO: u8 = 0x00;
pub const OP_MODE: u8 = 0x01;

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

// RegDioMapping2 ----------------------------------------------------------------------------------
pub const DIO_MAPPING_2: u8 = 0x41;
pub const DIO_MAPPING_2_DIO4_MASK: u8 = 0xc0;
pub const DIO_MAPPING_2_DIO4_SHIFT: u8 = 0x6;
pub const DIO_MAPPING_2_DIO5_MASK: u8 = 0x30;
pub const DIO_MAPPING_2_DIO5_SHIFT: u8 = 0x4;

// -------------------------------------------------------------------------------------------------
pub const VERSION: u8 = 0x42;
pub const TCXO: u8 = 0x4b;

// RegPaDac ----------------------------------------------------------------------------------------
pub const PA_DAC: u8 = 0x4d;
pub const PA_DAC_MASK: u8 = 0x07;

// -------------------------------------------------------------------------------------------------
pub const FORMER_TEMP: u8 = 0x5b;
pub const AGC_REF: u8 = 0x61;
pub const AGC_THRESH_1: u8 = 0x62;
pub const AGC_THRESH_2: u8 = 0x63;
pub const AGC_THRESH_3: u8 = 0x64;
pub const PLL: u8 = 0x70;