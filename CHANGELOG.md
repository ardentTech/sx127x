# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `lora` and `fsk` cargo features
- `valid_rx_headers` method
- `valid_rx_packets` method
- `last_packet_snr` method
- `set_invert_iq` method
- `bandwidth`, `coding_rate`, `spreading_factor` methods
- `symbol_rate`, `data_rate` methods
- make SPI `write` method public
- `set_temp_monitor` method
- `set_lna_gain` method
- `set_ocp` method
- `set_crc` method and make `set_coding_rate` public
- `rssi`, `last_packet_rssi` methods
- `set_low_data_rate_optimize` method
- `set_preamble_length` method
- `set_pa_boost`, `set_pa_rfo` methods
- `frequency_error_indication_hz`, `frequency_error_indication_ppm` methods
- `set_pa_ramp` method

## [0.1.0] - 2026-03-07

### Added
- async lora tx example
- async lora rx timeout example
- async lora rx continuous example
- async lora driver