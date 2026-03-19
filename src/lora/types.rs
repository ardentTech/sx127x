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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dio0Signal {
    RxDone = 0x0,
    TxDone = 0x1,
    CadDone = 0x2,
    None = 0x3,
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