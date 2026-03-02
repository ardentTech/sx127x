//! This example shows how to use the Lorem modem to transmit a packet using the rp235x-hal.

#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use {defmt_rtt as _, panic_probe as _};
use sx127x::{Sx127x, Sx127xError};

#[embassy_executor::main]
async fn main(_task_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_9, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_device = SpiDevice::new(&spi_bus, cs);
    let mut sx1276 = Sx127x::new(spi_device);

    match sx1276.read(0x18).await {
        Ok(byte) => info!("byte = {:02x}", byte),
        Err(err) => match err {
            Sx127xError::SPI(e) => match e {
                SpiDeviceError::Spi(_) => error!("Spi error"),
                SpiDeviceError::Cs(_) => error!("Cs error"),
                SpiDeviceError::DelayNotSupported => error!("DelayNotSupported error"),
                SpiDeviceError::Config => error!("Config error"),
                _ => error!("Unexpected error"),
            }
        }
    }

    loop {
        info!("looping");
        embassy_time::Timer::after(embassy_time::Duration::from_millis(3_000)).await;
    }
}