#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ContinuousDio0Signal {
    #[default]
    SyncAddressOrTxReady = 0x0,
    RssiPreambleDetect = 0x1,
    RxReadyTxReady = 0x2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ContinuousDio1Signal {
    #[default]
    Dclk = 0x0,
    RssiPreambleDetect = 0x1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ContinuousDio2Signal {
    #[default]
    Data = 0x0,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ContinuousDio3Signal {
    #[default]
    TimeOut = 0x0,
    RssiPreambleDetect = 0x1,
    TempChangeLowBat = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ContinuousDio4Signal {
    #[default]
    TempChangeLowBat = 0x0,
    PllLock = 0x1,
    TimeOut = 0x2,
    ModeReady = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ContinuousDio5Signal {
    #[default]
    ClkOut = 0x0,
    PllLock = 0x1,
    RssiPreambleDetect = 0x2,
    ModeReady = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketDio0Signal {
    #[default]
    PayloadReadyOrPacketSent = 0x0,
    CrcOk = 0x1,
    TempChangeLowBat = 0x2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketDio1Signal {
    #[default]
    FifoLevel = 0x0,
    FifoEmpty = 0x1,
    FifoFull = 0x2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketDio2Signal {
    #[default]
    FifoFull = 0x0,
    RxReady = 0x1,
    FifoFullOrTimeOut = 0x2,
    FifoFullOrSyncAddress = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketDio3Signal {
    #[default]
    FifoEmpty = 0x0,
    TxReady = 0x1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketDio4Signal {
    #[default]
    TempChangeLowBat = 0x0,
    PllLock = 0x1,
    TimeOut = 0x2,
    RssiPreambleDetect = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PacketDio5Signal {
    #[default]
    ClkOut = 0x0,
    PllLock = 0x1,
    Data = 0x2,
    ModeReady = 0x3,
}