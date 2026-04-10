#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bandwidth {
    Bw2_6kHz,
    Bw3_1kHz,
    Bw3_9kHz,
    Bw5_2kHz,
    Bw6_3kHz,
    Bw7_8kHz,
    #[default]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModulationType {
    FSK = 0x00,
    OOK = 0x01
}