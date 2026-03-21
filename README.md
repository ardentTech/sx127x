# Sx127x
`#![no_std]`, `async`-first driver for the Semtech SX127X transceiver built on top of Rust [embedded-hal](https://github.com/rust-embedded/embedded-hal).

## Overview
TODO describe modems

### Cargo Features
- `lora` (default): compile the LoRa modem
- `fsk`: compile the FSK/OOK modem
- `async` (default): compile async impls
- `sync`: compile sync impls

### Reset
- "A power-on reset of the SX1276/77/78/79 is triggered at power up." [datasheet section 5.2]
- To perform a manual reset: "Pin 7 should be pulled low for a hundred microseconds, and then released. The user should then wait for 5 ms before using
  the chip." [datasheet section 5.2.2]

### TODO
- [ ] `async` LoRa impl
- [ ] `sync` LoRa impl
- [ ] `async` FSK/OOK impl
- [ ] `sync` FSK/OOK impl
- [ ] lora errata 2.3
- [ ] lora errata 2.4
- [ ] fsk errata 3.1
- [ ] fsk errata 3.2
- [ ] RX, RX_TX, TX driver modes to customize FIFO buffer mem layout