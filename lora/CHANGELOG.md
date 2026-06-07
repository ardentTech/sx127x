# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `crc` getter
- `half_duplex` example

### Changed

- move IQ inversion out of `RxConfig` and `TxConfig` into dedicated `set_iq_inversion` driver method.
- move preamble set to `Sx127xLoraConfig`
- make `optimize_rx_response` public

### Removed

- `RxConfig` struct
- `config_rx` method