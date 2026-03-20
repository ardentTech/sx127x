//! This async example reads the calculated LoRa values symbol rate (chips/s) and data rate (bits/s).
#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use sx127x::lora::driver::{Sx127xConfig, Sx127xLora};

const FREQUENCY_HZ: u32 = 915_000_000;

#[embassy_executor::main]
async fn main(_task_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut config = Sx127xConfig::default();
    config.frequency = FREQUENCY_HZ;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.expect("driver init failed :(");

    let symbol_rate = sx127x.symbol_rate().await.expect("failed to get symbol_rate :(");
    info!("symbol_rate expected: {}, actual: {}", 976, symbol_rate);

    let data_rate = sx127x.data_rate().await.expect("failed to get data_rate :(");
    info!("data_rate expected: {}, actual: {}", 5465, data_rate);

    loop {
        Timer::after(embassy_time::Duration::from_millis(3_000)).await;
        info!("looping around");
    }
}