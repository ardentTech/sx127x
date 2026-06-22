//! This example reads the raw, uncalibrated temperature in °C.
#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};
use sx127xfsk::data_mode::PacketMode;
use sx127xfsk::driver::Sx127xFsk;

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::main]
async fn main(_task_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut fsk: Sx127xFsk<PacketMode, SpiDevice<'_, _, Spi<'_, _, _>, Output<'_>>> = Sx127xFsk::new(spi_dev).await.expect("driver init failed :(");
    let mut temp = fsk.raw_temperature(Delay).await.unwrap();

    loop {
        info!("raw (uncalibrated) temperature: {}°C", temp);
        Timer::after(embassy_time::Duration::from_millis(3_000)).await;
        temp = fsk.raw_temperature(Delay).await.unwrap();
    }
}