//! This example reads a random byte from the chip.
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
use embassy_time::Timer;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::{ex_config, led_task};
use sx127xlora::driver::{Sx127xLora};
use sx127xlora::types::{CadDone, PowerRamp, TxConfig, TxDone, OCP};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut sx127x = Sx127xLora::new(spi_dev, ex_config()).await.unwrap();
    sx127x.configure_tx(TxConfig::new(OCP::default(), 20, PowerRamp::default(), false).unwrap()).await.unwrap();

    sx127x.map_dio0::<TxDone>().await.unwrap();
    sx127x.map_dio3::<CadDone>().await.unwrap();

    spawner.spawn(led_task(Output::new(p.PIN_9, Level::Low), Output::new(p.PIN_7, Level::Low)).unwrap());
    sx127x.random().await.unwrap();

    loop {
        info!("random: {}", sx127x.random().await.unwrap());
        Timer::after_millis(3_000).await;
    }
}