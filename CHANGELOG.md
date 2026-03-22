# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `lora` and `fsk` cargo features
- `data_rate` and `symbol_rate` calculators and methods
- `calibrate`, `crc_generation` methods
- `clear_interrupt`, `mask_interrupt` method and `Interrupt` enum
- `bandwidth`, `set_bandwidth` methods and `Bandwidth` enum
- `coding_rate`, `set_coding_rate` methods and `CodingRate` enum
- `device_mode`, `set_device_mode` methods and `DeviceMode` enum
- `frequency`, `set_frequency` methods
- `header_mode`, `set_header_mode` methods and `HeaderMode` enum
- `modem_status` method and `ModemStatus` enum
- `read`, `read_rx_data` methods
- `receive`, `transmit` methods
- `set_dio0`, `set_dio1` methods and `Dio0Signal`, `Dio1Signal` enums
- `set_temp_monitor` method
- `spreading_factor`, `set_spreading_factor` methods and `SpreadingFactor` enum

## [0.1.0] - 2026-03-07

### Added
- async lora tx example
- async lora rx timeout example
- async lora rx continuous example
- async lora driver shell