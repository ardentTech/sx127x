# Sx127x
`#![no_std]`, `async`-first drivers for the Semtech SX127X transceiver built on top of Rust [embedded-hal](https://github.com/rust-embedded/embedded-hal).

## Crates

| Crate                                     | Dir          | Docs                                            | Description                  |
|-------------------------------------------|--------------|-------------------------------------------------|------------------------------|
| [sx127x](https://crates.io/crates/sx127x) | ./dual_modem | [Docs.rs](https://docs.rs/sx127x/0.1.0/sx127x/) | Dual-modem (FSK + LoRa) impl |
| sx127x-common                             | ./common     | -                                               | code common to both modems   |
| sx127x-fsk                                | ./fsk        | -                                               | FSK modem impl               |
| sx127x-lora                               | ./lora       | -                                               | LoRa modem impl              |