pub use sx127x_common::registers::*;

// -------------------------------------------------------------------------------------------------
pub const BITRATE_MSB: u8 = 0x02;
pub const BITRATE_LSB: u8 = 0x03;
pub const FDEV_MSB: u8 = 0x04;
pub const FDEV_LSB: u8 = 0x05;

// RegRxConfig -------------------------------------------------------------------------------------
pub const RX_CONFIG: u8 = 0x0d;
pub const RX_CONFIG_RESTART_RX_ON_COLLISION_MASK: u8 = 0x80;
pub const RX_CONFIG_RESTART_RX_ON_COLLISION_OFFSET: u8 = 0x7;
pub const RX_CONFIG_RESTART_WITHOUT_PLL_LOCK_MASK: u8 = 0x40;
pub const RX_CONFIG_RESTART_WITHOUT_PLL_LOCK_OFFSET: u8 = 0x6;
pub const RX_CONFIG_RESTART_WITH_PLL_LOCK_MASK: u8 = 0x20;
pub const RX_CONFIG_RESTART_WITH_PLL_LOCK_OFFSET: u8 = 0x5;
pub const RX_CONFIG_AFC_AUTO_ON_MASK: u8 = 0x10;
pub const RX_CONFIG_AFC_AUTO_ON_OFFSET: u8 = 0x4;
pub const RX_CONFIG_AGC_AUTO_ON_MASK: u8 = 0x08;
pub const RX_CONFIG_AGC_AUTO_ON_OFFSET: u8 = 0x3;
pub const RX_CONFIG_RX_TRIGGER_MASK: u8 = 0x07;
pub const RX_CONFIG_RX_TRIGGER_OFFSET: u8 = 0x0;

// RegRssiConfig -----------------------------------------------------------------------------------
pub const RSSI_CONFIG: u8 = 0x0e;
pub const RSSI_CONFIG_RSSI_OFFSET_MASK: u8 = 0xf8;
pub const RSSI_CONFIG_RSSI_OFFSET_OFFSET: u8 = 0x3;
pub const RSSI_CONFIG_RSSI_SMOOTHING_MASK: u8 = 0x07;
pub const RSSI_CONFIG_RSSI_SMOOTHING_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const RSSI_COLLISION: u8 = 0x0f;
pub const RSSI_THRESH: u8 = 0x10;
pub const RSSI_VALUE: u8 = 0x11;

// RegRxBw -----------------------------------------------------------------------------------------
pub const RX_BW: u8 = 0x12;
pub const RX_BW_MANT_MASK: u8 = 0x18;
pub const RX_BW_MANT_OFFSET: u8 = 0x3;
pub const RX_BW_EXP_MASK: u8 = 0x07;
pub const RX_BW_EXP_OFFSET: u8 = 0x0;

// RegAfcBw ----------------------------------------------------------------------------------------
pub const AFC_BW: u8 = 0x13;
pub const AFC_BW_MANT_MASK: u8 = 0x18;
pub const AFC_BW_MANT_OFFSET: u8 = 0x3;
pub const AFC_BW_EXP_MASK: u8 = 0x07;
pub const AFC_BW_EXP_OFFSET: u8 = 0x0;

// RegOokPeak --------------------------------------------------------------------------------------
pub const OOK_PEAK: u8 = 0x14;
pub const OOK_PEAK_BIT_SYNC_ON_MASK: u8 = 0x20;
pub const OOK_PEAK_BIT_SYNC_ON_OFFSET: u8 = 0x5;
pub const OOK_PEAK_OOK_THRESH_TYPE_MASK: u8 = 0x18;
pub const OOK_PEAK_OOK_THRESH_TYPE_OFFSET: u8 = 0x3;
pub const OOK_PEAK_OOK_PEAK_THRESH_STEP_MASK: u8 = 0x07;
pub const OOK_PEAK_OOK_PEAK_THRESH_STEP_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const OOK_FIX: u8 = 0x15;

// RegOokAvg ---------------------------------------------------------------------------------------
pub const OOK_AVG: u8 = 0x16;
pub const OOK_AVG_OOK_PEAK_THRESH_DEC_MASK: u8 = 0xe0;
pub const OOK_AVG_OOK_PEAK_THRESH_DEC_OFFSET: u8 = 0x5;
pub const OOK_AVG_OOK_AVERAGE_OFFSET_MASK: u8 = 0x0c;
pub const OOK_AVG_OOK_AVERAGE_OFFSET_OFFSET: u8 = 0x2;
pub const OOK_AVG_OOK_AVERAGE_THRESH_FILT_MASK: u8 = 0x03;
pub const OOK_AVG_OOK_AVERAGE_THRESH_FILT_OFFSET: u8 = 0x0;

// RegAfcFei ---------------------------------------------------------------------------------------
pub const AFC_FEI: u8 = 0x1a;
pub const AFC_FEI_AGC_START_MASK: u8 = 0x10;
pub const AFC_FEI_AGC_START_OFFSET: u8 = 0x4;
pub const AFC_FEI_AFC_CLEAR_MASK: u8 = 0x02;
pub const AFC_FEI_AFC_CLEAR_OFFSET: u8 = 0x1;
pub const AFC_FEI_AFC_AUTO_CLEAR_ON_MASK: u8 = 0x01;
pub const AFC_FEI_AFC_AUTO_CLEAR_ON_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const AFC_MSB: u8 = 0x1b;
pub const AFC_LSB: u8 = 0x1c;
pub const FEI_MSB: u8 = 0x1d;
pub const FEI_LSB: u8 = 0x1e;

// RegPreambleDetect -------------------------------------------------------------------------------
pub const PREAMBLE_DETECT: u8 = 0x1f;
pub const PREAMBLE_DETECT_PREAMBLE_DETECTOR_ON_MASK: u8 = 0x80;
pub const PREAMBLE_DETECT_PREAMBLE_DETECTOR_ON_OFFSET: u8 = 0x7;
pub const PREAMBLE_DETECT_PREAMBLE_DETECTOR_SIZE_MASK: u8 = 0x60;
pub const PREAMBLE_DETECT_PREAMBLE_DETECTOR_SIZE_OFFSET: u8 = 0x5;
pub const PREAMBLE_DETECT_PREAMBLE_DETECTOR_TOL_MASK: u8 = 0x1f;
pub const PREAMBLE_DETECT_PREAMBLE_DETECTOR_TOL_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const RX_TIMEOUT_1: u8 = 0x20;
pub const RX_TIMEOUT_2: u8 = 0x21;
pub const RX_TIMEOUT_3: u8 = 0x22;
pub const RX_DELAY: u8 = 0x23;

// RegOsc ------------------------------------------------------------------------------------------
pub const OSC: u8 = 0x24;
pub const OSC_RC_CAL_START_MASK: u8 = 0x08;
pub const OSC_RC_CAL_START_OFFSET: u8 = 0x3;
pub const OSC_CLK_OUT_MASK: u8 = 0x07;
pub const OSC_CLK_OUT_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const PREAMBLE_MSB: u8 = 0x25;
pub const PREAMBLE_LSB: u8 = 0x26;

// RegSyncConfig -----------------------------------------------------------------------------------
pub const SYNC_CONFIG: u8 = 0x27;
pub const SYNC_CONFIG_AUTO_RESTART_RX_MODE_MASK: u8 = 0xc0;
pub const SYNC_CONFIG_AUTO_RESTART_RX_MODE_OFFSET: u8 = 0x6;
pub const SYNC_CONFIG_PREAMBLE_POLARITY_MASK: u8 = 0x20;
pub const SYNC_CONFIG_PREAMBLE_POLARITY_OFFSET: u8 = 0x5;
pub const SYNC_CONFIG_SYNC_ON_MASK: u8 = 0x10;
pub const SYNC_CONFIG_SYNC_ON_OFFSET: u8 = 0x4;
pub const SYNC_CONFIG_SYNC_SIZE_MASK: u8 = 0x07;
pub const SYNC_CONFIG_SYNC_SIZE_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const SYNC_VALUE_1: u8 = 0x28;

// RegPacketConfig1 --------------------------------------------------------------------------------
pub const PACKET_CONFIG_1: u8 = 0x30;
pub const PACKET_CONFIG_1_PACKET_FORMAT_MASK: u8 = 0x80;
pub const PACKET_CONFIG_1_PACKET_FORMAT_OFFSET: u8 = 0x7;
pub const PACKET_CONFIG_1_DC_FREE_MASK: u8 = 0x60;
pub const PACKET_CONFIG_1_DC_FREE_OFFSET: u8 = 0x5;
pub const PACKET_CONFIG_1_CRC_ON_MASK: u8 = 0x10;
pub const PACKET_CONFIG_1_CRC_ON_OFFSET: u8 = 0x4;
pub const PACKET_CONFIG_1_CRC_AUTO_CLEAR_OFF_MASK: u8 = 0x08;
pub const PACKET_CONFIG_1_CRC_AUTO_CLEAR_OFF_OFFSET: u8 = 0x3;
pub const PACKET_CONFIG_1_ADDRESS_FILTERING_MASK: u8 = 0x06;
pub const PACKET_CONFIG_1_ADDRESS_FILTERING_OFFSET: u8 = 0x1;
pub const PACKET_CONFIG_1_CRC_WHITENING_TYPE_MASK: u8 = 0x01;
pub const PACKET_CONFIG_1_CRC_WHITENING_TYPE_OFFSET: u8 = 0x0;

// RegPacketConfig2 --------------------------------------------------------------------------------
pub const PACKET_CONFIG_2: u8 = 0x31;
pub const PACKET_CONFIG_2_DATA_MODE_MASK: u8 = 0x40;
pub const PACKET_CONFIG_2_DATA_MODE_OFFSET: u8 = 0x6;
pub const PACKET_CONFIG_2_IO_HOME_ON_MASK: u8 = 0x20;
pub const PACKET_CONFIG_2_IO_HOME_ON_OFFSET: u8 = 0x5;
pub const PACKET_CONFIG_2_IO_HOME_POWER_FRAME_MASK: u8 = 0x10;
pub const PACKET_CONFIG_2_IO_HOME_POWER_FRAME_OFFSET: u8 = 0x4;
pub const PACKET_CONFIG_2_BEACON_ON_MASK: u8 = 0x08;
pub const PACKET_CONFIG_2_BEACON_ON_OFFSET: u8 = 0x3;
pub const PACKET_CONFIG_2_PAYLOAD_LENGTH_MASK: u8 = 0x07;
pub const PACKET_CONFIG_2_PAYLOAD_LENGTH_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const PAYLOAD_LENGTH: u8 = 0x32;
pub const NODE_ADRS: u8 = 0x33;
pub const BROADCAST_ADRS: u8 = 0x34;

// RegFifoThresh -----------------------------------------------------------------------------------
pub const FIFO_THRESH: u8 = 0x35;
pub const FIFO_THRESH_TX_START_CONDITION_MASK: u8 = 0x80;
pub const FIFO_THRESH_TX_START_CONDITION_OFFSET: u8 = 0x7;
pub const FIFO_THRESH_FIFO_THRESHOLD_MASK: u8 = 0x3f;
pub const FIFO_THRESH_FIFO_THRESHOLD_OFFSET: u8 = 0x0;

// RegSeqConfig1 -----------------------------------------------------------------------------------
pub const SEQ_CONFIG_1: u8 = 0x36;
pub const SEQ_CONFIG_1_SEQUENCER_START_MASK: u8 = 0x80;
pub const SEQ_CONFIG_1_SEQUENCER_START_OFFSET: u8 = 0x7;
pub const SEQ_CONFIG_1_SEQUENCER_STOP_MASK: u8 = 0x40;
pub const SEQ_CONFIG_1_SEQUENCER_STOP_OFFSET: u8 = 0x6;
pub const SEQ_CONFIG_1_IDLE_MODE_MASK: u8 = 0x20;
pub const SEQ_CONFIG_1_IDLE_MODE_OFFSET: u8 = 0x5;
pub const SEQ_CONFIG_1_FROM_START_MASK: u8 = 0x18;
pub const SEQ_CONFIG_1_FROM_START_OFFSET: u8 = 0x3;
pub const SEQ_CONFIG_1_LOW_POWER_SELECTION_MASK: u8 = 0x04;
pub const SEQ_CONFIG_1_LOW_POWER_SELECTION_OFFSET: u8 = 0x2;
pub const SEQ_CONFIG_1_FROM_IDLE_MASK: u8 = 0x02;
pub const SEQ_CONFIG_1_FROM_IDLE_OFFSET: u8 = 0x1;
pub const SEQ_CONFIG_1_FROM_TRANSMIT_MASK: u8 = 0x01;
pub const SEQ_CONFIG_1_FROM_TRANSMIT_OFFSET: u8 = 0x0;

// RegSeqConfig2 -----------------------------------------------------------------------------------
pub const SEQ_CONFIG_2: u8 = 0x37;
pub const SEQ_CONFIG_2_FROM_RECEIVE_MASK: u8 = 0xe0;
pub const SEQ_CONFIG_2_FROM_RECEIVE_OFFSET: u8 = 0x5;
pub const SEQ_CONFIG_2_FROM_RX_TIMEOUT_MASK: u8 = 0x18;
pub const SEQ_CONFIG_2_FROM_RX_TIMEOUT_OFFSET: u8 = 0x3;
pub const SEQ_CONFIG_2_FROM_PACKET_RECEIVED_MASK: u8 = 0x07;
pub const SEQ_CONFIG_2_FROM_PACKET_RECEIVED_OFFSET: u8 = 0x0;

// RegTimerResol -----------------------------------------------------------------------------------
pub const TIMER_RESOL: u8 = 0x38;
pub const TIMER_RESOL_TIMER_1_RESOLUTION_MASK: u8 = 0x0c;
pub const TIMER_RESOL_TIMER_1_RESOLUTION_OFFSET: u8 = 0x2;
pub const TIMER_RESOL_TIMER_2_RESOLUTION_MASK: u8 = 0x03;
pub const TIMER_RESOL_TIMER_2_RESOLUTION_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const TIMER_1_COEFF: u8 = 0x39;
pub const TIMER_2_COEFF: u8 = 0x3a;

// RegImageCal -------------------------------------------------------------------------------------
pub const IMAGE_CAL: u8 = 0x3b;
pub const IMAGE_CAL_AUTO_IMAGE_CAL_ON_MASK: u8 = 0x80;
pub const IMAGE_CAL_AUTO_IMAGE_CAL_ON_OFFSET: u8 = 0x7;
pub const IMAGE_CAL_IMAGE_CAL_START_MASK: u8 = 0x40;
pub const IMAGE_CAL_IMAGE_CAL_START_OFFSET: u8 = 0x6;
pub const IMAGE_CAL_IMAGE_CAL_RUNNING_MASK: u8 = 0x20;
pub const IMAGE_CAL_IMAGE_CAL_RUNNING_OFFSET: u8 = 0x5;
pub const IMAGE_CAL_TEMP_CHANGE_MASK: u8 = 0x08;
pub const IMAGE_CAL_TEMP_CHANGE_OFFSET: u8 = 0x3;
pub const IMAGE_CAL_TEMP_THRESHOLD_MASK: u8 = 0x06;
pub const IMAGE_CAL_TEMP_THRESHOLD_OFFSET: u8 = 0x1;
pub const IMAGE_CAL_TEMP_MONITOR_OFF_MASK: u8 = 0x01;
pub const IMAGE_CAL_TEMP_MONITOR_OFF_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const TEMP: u8 = 0x3c;

// RegLowBat ---------------------------------------------------------------------------------------
pub const LOW_BAT: u8 = 0x3d;
pub const LOW_BAT_ON_MASK: u8 = 0x08;
pub const LOW_BAT_ON_OFFSET: u8 = 0x3;
pub const LOW_BAT_TRIM_MASK: u8 = 0x07;
pub const LOW_BAT_TRIM_OFFSET: u8 = 0x0;

// -------------------------------------------------------------------------------------------------
pub const IRQ_FLAGS_1: u8 = 0x3e;
pub const IRQ_FLAGS_1_MODE_READY_MASK: u8 = 0x80;
pub const IRQ_FLAGS_1_MODE_READY_OFFSET: u8 = 0x7;

// TODO pub const IRQ_FLAGS_2: u8 = 0x3f;

// RegPllHop ---------------------------------------------------------------------------------------
pub const PLL_HOP: u8 = 0x44;
pub const PLL_HOP_FAST_HOP_ON_MASK: u8 = 0x80;
pub const PLL_HOP_FAST_HOP_ON_OFFSET: u8 = 0x7;

// -------------------------------------------------------------------------------------------------
pub const BITRATE_FRAC: u8 = 0x5d;
pub const BITRATE_FRAC_MASK: u8 = 0xf;
pub const BITRATE_FRAC_OFFSET: u8 = 0x0;