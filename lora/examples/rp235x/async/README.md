# RP235x Async Examples

These examples were developed and validated with a [Pico 2 W](https://www.raspberrypi.com/products/raspberry-pi-pico-2/) and the [Adafruit RFM95W](https://www.adafruit.com/product/3072) breakout board. For
different hardware combinations, it's important to select the correct frequency for your region and verify the pin
mappings.

As the driver is compiled **without** the `half-duplex` feature for these examples, it defaults to full-duplex mode with
two 128 byte buffers, one for RX and another for TX.

## Usage

1. Set log level as needed: `$ export DEFMT_LOG=info`
2. Run example: `$ cargo run --bin tx`
