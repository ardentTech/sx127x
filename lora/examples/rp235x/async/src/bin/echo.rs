//! This example will echo whatever packet payload it receives. The green led will pulse on
//! `RxDone` and the red led will pulse on `TxDone`.
#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::{ex_config, led_task, Led, PULSE_LED};
use sx127xlora::driver::Sx127xLora;
use sx127xlora::types::{RxDone, TxDone};

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

    let mut dio0 = Input::new(p.PIN_15, Pull::Down);

    let mut sx127x = Sx127xLora::new_with_config(spi_dev, ex_config()).await.unwrap();

    spawner.spawn(led_task(Output::new(p.PIN_9, Level::Low), Output::new(p.PIN_7, Level::Low)).unwrap());

    loop {
        sx127x.map_dio0::<RxDone>().await.unwrap();
        sx127x.rx(None).await.unwrap();
        dio0.wait_for_high().await;
        sx127x.clear_interrupt::<RxDone>().await.unwrap();
        match sx127x.rx_packet().await {
            Ok(rxp) => {
                let len: usize = rxp.payload.iter().filter(|c| **c != 0).count();
                PULSE_LED.signal(Led::Green);

                sx127x.map_dio0::<TxDone>().await.unwrap();
                sx127x.tx(&rxp.payload).await.unwrap();
                dio0.wait_for_high().await;
                sx127x.clear_interrupt::<TxDone>().await.unwrap();
                PULSE_LED.signal(Led::Red);
            }
            Err(_) => error!("rx_packet() failed :(")
        }
    }
}