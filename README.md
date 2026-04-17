# Sx127x
`#![no_std]`, `async`-first driver for using both the FSK and LoRa modems on the Semtech SX127X transceiver. As it
depends on both the [sx127x-fsk](https://github.com/ardentTech/sx127x-fsk) and [sx127x-lora](https://github.com/ardentTech/sx127x-lora)
crates, it has a larger footprint than either modem driver on its own.

### Cargo Features

- `defmt`: include deferred formatting logging functionality
- `sync`: modem sync implementation

### Resources

* [Datasheet](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R0000001Rbr/6EfVZUorrpoKFfvaF_Fkpgp5kzjiNyiAbqcpqh9qSjE)
* [Errata](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R000000HSPv/sqi9xX0gs6hgzl2LoPwCK0TS9GDPlMwsXmcNzJCMHjw)

### License

* [MIT](https://github.com/ardentTech/sx127x-lora/blob/main/LICENSE-MIT)
* [Apache](https://github.com/ardentTech/sx127x-lora/blob/main/LICENSE-APACHE)