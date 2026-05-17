# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `set_bandwidth`, `set_device_mode` methods
- `bit_rate`, `set_bit_rate` methods
- `fdev`, `set_fdev` methods
- `set_frequency`, `set_rx_config` methods
- `RssiSmoothing` enum and `set_rssi_smoothing` method
- `restart_rx` method
- `set_rssi_offset` method
- `set_rssi_collision_threshold`, `set_rssi_threshold` methods
- `set_afc_bw` method
- `set_inter_packet_rx_delay`, `set_ook_threshold` methods
- `set_ook_peak_config` method
- `set_ook_avg` method
- `set_afc_auto_clear`, `start_agc_sequence` methods
- `set_afc` method
- `fei`, `set_fei` methods
- `set_clk_out`, `calibrate_rc_oscillator` methods
- `set_preamble_size`, `preamble_size` methods
- `set_sync_config` method
- `set_sync_values` method
- `payload_length`, `set_payload_length` methods
- `node_addr`, `set_node_addr` methods
- `broadcast_addr`, `set_broadcast_addr` methods
- `fifo_threshold`, `set_fifo_threshold`, `tx_start_condition`, `set_tx_start_condition` methods
- `timer1_coefficient`, `set_timer1_coefficient`, `timer2_coefficient`, `set_timer2_coefficient` methods
- `temp` method
- `set_low_battery_detector`, `set_low_battery_trim` methods
- `version` method
- `start_image_calibration`, `temp_change_greater_than_threshold`, `temp_threshold`, `set_temp_threshold` methods
- `auto_image_calibration`, `set_auto_image_calibration` methods
- `start_sequencer`, `stop_sequencer` methods
- `set_timer1`, `set_timer2` methods
- `image_calibration_running` method
- `set_sequencer_transitions` method
- `sequencer_transitions` method
- `set_preamble_detector`, `preamble_detector` methods
- `set_signal_sync_timeout`, `set_rx_preamble_timeout`, `set_rx_rssi_timeout` methods
- `packet_config`, `set_packet_config` methods
- `DataMode` trait and `set_dioX` methods
