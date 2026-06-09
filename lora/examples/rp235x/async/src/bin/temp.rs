//! This example demonstrates CAD and TX by checking for channel activity before transmitting a 128 byte payload. The green led on GPIO 21 will pulse on
//! success, or the red les on GPIO 22 will pulse on error.
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
use embassy_time::{Delay, Timer};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::debug_config;
use sx127xlora::driver::{Sx127xLora};

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

    let mut sx127x = Sx127xLora::new(spi_dev, debug_config(), Delay).await.unwrap();
    let mut temp = sx127x.measure_temp(Delay).await.unwrap();

    loop {
        info!("temp: {}°C", temp);
        info!("temp: {}°F", (temp * 9/5) + 32);
        Timer::after_millis(1_000).await;
        temp = sx127x.measure_temp(Delay).await.unwrap();
    }
}