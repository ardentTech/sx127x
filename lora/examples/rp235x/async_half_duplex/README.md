# RP235x Async Half Duplex Examples

These examples were developed and validated with a
[Pico 2 W](https://www.raspberrypi.com/documentation/microcontrollers/images/pico2w-pinout.svg) and the
[Adafruit RFM95W](https://www.adafruit.com/product/3072) breakout board. For different hardware combinations, it's
important to select the correct frequency for your region and verify the pin mappings.

As the driver is compiled **with** the `half-duplex` feature for these examples, it features a single 256 byte buffers
for RX or TX.

## Usage

1. Set log level as needed: `$ export DEFMT_LOG=info`
2. Run example: `$ cargo run --bin tx`
