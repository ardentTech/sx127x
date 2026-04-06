# Sx127x
`#![no_std]`, `async`-first driver for the Semtech SX127X transceiver built on top of Rust [embedded-hal](https://github.com/rust-embedded/embedded-hal).

### Cargo Features
- `lora` (default): LoRa modem
- `fsk`: FSK/OOK modem
- `async` (default): modem async implementation
- `sync`: modem sync implementation
- `half_duplex`: use the full data buffer size of 256 bytes for RX or TX, at the cost of full duplex RX and TX with 128 byte buffers.

### Roadmap
- [ ] LoRa async (in-progress)
- [ ] LoRa sync
- [ ] FSK async
- [ ] FSK sync

### Examples
* [LoRa RP235x async](https://github.com/ardentTech/sx127x/tree/main/examples/rp235x/lora/async)

### Resources
* [Datasheet](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R0000001Rbr/6EfVZUorrpoKFfvaF_Fkpgp5kzjiNyiAbqcpqh9qSjE)

### License
* [MIT](https://github.com/ardentTech/sx127x/blob/main/LICENSE-MIT)
* [Apache](https://github.com/ardentTech/sx127x/blob/main/LICENSE-APACHE)