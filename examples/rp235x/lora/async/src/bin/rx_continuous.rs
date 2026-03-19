//! This async example demonstrates how to use the LoRa modem in RXCONTINUOUS mode and handle the
//! RxDone interrupt on DIO0. You will need a second sx127x chip in range and with the same settings
//! to handle tx.
#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use {defmt_rtt as _, panic_probe as _};
use sx127x::lora::driver::{Sx127xSpi, Sx127xConfig};
use sx127x::lora::types::{Dio0, Interrupt};

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

    let mut dio0 = Input::new(p.PIN_15, Pull::Down);

    let mut config = Sx127xConfig::default();
    config.frequency = FREQUENCY_HZ;
    let mut sx127x = Sx127xSpi::new(spi_dev, config).await.expect("driver init failed :(");

    sx127x.enable_dio0(Dio0::RxDone).await.expect("enable_dio0 failed :(");
    sx127x.receive(None).await.expect("receive failed :(");

    loop {
        dio0.wait_for_high().await;
        info!("RxDone triggered!");
        sx127x.clear_interrupt(Interrupt::RxDone).await.expect("clear interrupt RxDone failed :(");
        match sx127x.read_rx_data().await {
            Ok(buf) => {
                let len: usize = buf.iter().filter(|c| **c != 0).count();
                info!("rx buffer: {:a}", buf[..len])
            },
            Err(_) => error!("read_rx_data failed :(")
        }
        info!("looping around");
    }
}