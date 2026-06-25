//! This example implicitly and explicitly tests the SPI peripheral on the sx127x module. Implicitly, the call to Sx127xLora::new(...) will read and verify the
//! value of RegVersion and then write the Sx127xLoraConfig members to their respective registers. If this passes, then this firmware will explicitly write to
//! and read from the RxPayloadCrcOn bit of the RegModemConfig2 register.
#![no_std]
#![no_main]

use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::ex_config;
use sx127xlora::driver::Sx127xLora;

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut config = ex_config();
    config.use_crc = false;
    let mut sx127x = Sx127xLora::new_with_config(spi_dev, config).await.unwrap();

    sx127x.set_crc(true).await.unwrap();
    assert!(sx127x.crc().await.unwrap());
    sx127x.set_crc(false).await.unwrap();
    assert!(!sx127x.crc().await.unwrap());
    info!("SPI read/write test passed");

    loop {}
}