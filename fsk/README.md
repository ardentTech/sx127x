# Sx127x-LoRa
`#![no_std]`, `async`-first driver for the FSK/OOK modem on the Semtech SX127X transceiver built on top of Rust [embedded-hal](https://github.com/rust-embedded/embedded-hal).

### Cargo Features
- `async` (default): modem async implementation
- `sync`: modem sync implementation

### Roadmap
- [ ] async (in-progress)
- [ ] sync

### Examples
TODO

### TODO
- [ ] rename `sx127x-fsk` to `sx127x-fskook`?
- [ ] LowFrequencyModeOn bit of RegOpMode

### Resources
* [Datasheet](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R0000001Rbr/6EfVZUorrpoKFfvaF_Fkpgp5kzjiNyiAbqcpqh9qSjE)
* [Errata](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R000000HSPv/sqi9xX0gs6hgzl2LoPwCK0TS9GDPlMwsXmcNzJCMHjw)

### License
* [MIT](https://github.com/ardentTech/sx127x/blob/main/LICENSE-MIT)
* [Apache](https://github.com/ardentTech/sx127x/blob/main/LICENSE-APACHE)