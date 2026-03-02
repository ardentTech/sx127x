# Sx127x
`#![no_std]`, `async`-first driver for the Semtech SX127X transceiver built on top of Rust [embedded-hal](https://github.com/rust-embedded/embedded-hal).

### Cargo Features
* `lora` (default): compile the LoRa modem
* `fsk`: compile the FSK/OOK modem
* `async` (default): compile the async library
* `sync`: compile the sync library

### Reset
* "A power-on reset of the SX1276/77/78/79 is triggered at power up." [section 5.2]
* To perform a manual reset: "Pin 7 should be pulled low for a hundred microseconds, and then released. The user should then wait for 5 ms before using
  the chip." [section 5.2.2]

### TODO
- [ ] `async` LoRa impl
- [ ] `sync` LoRa impl
- [ ] `async` FSK/OOK impl
- [ ] `sync` FSK/OOK impl
- [ ] full rx/tx mode to maximize FIFO buffer