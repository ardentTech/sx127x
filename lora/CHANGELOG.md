# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Move IQ inversion out of `RxConfig` and `TxConfig` into dedicated `set_iq_inversion` driver method. It appears that TX path IQ inversion defaults to on, which was breaking all examples.