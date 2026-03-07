# Sx127x
`#![no_std]`, `async`-first driver for the Semtech SX127X transceiver built on top of Rust [embedded-hal](https://github.com/rust-embedded/embedded-hal).

### Vs. sx127x_lora
- single source of truth
- async support
- compile for LoRa or FSK/OOK

### Cargo Features
- `lora` (default): compile the LoRa modem
- `fsk`: compile the FSK/OOK modem
- `async` (default): compile the async library
- `sync`: compile the sync library

### Reset
- "A power-on reset of the SX1276/77/78/79 is triggered at power up." [DS section 5.2]
- To perform a manual reset: "Pin 7 should be pulled low for a hundred microseconds, and then released. The user should then wait for 5 ms before using
  the chip." [DS section 5.2.2]

### TODO
- [ ] `async` LoRa impl
- [ ] `sync` LoRa impl
- [ ] `async` FSK/OOK impl
- [ ] `sync` FSK/OOK impl
- [ ] RX, RX_TX, TX driver modes to customize FIFO buffer mem layout
- [ ] set up `cargo-embed` for examples