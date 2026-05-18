//! This example demonstrates how to use the LoRa modem in RXSINGLE mode with a timeout. The
//! RxDone interrupt on DIO0 is wired to pin 15, and the RxTimeout interrupt on DIO1 is wired to pin
//! 14. You will need a second dual_modem chip in range and with the same settings to handle tx before
//! the timeout.
#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use {defmt_rtt as _, panic_probe as _};
use common::{heartbeat, LORA_FREQUENCY_HZ};
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{RxDone, RxTimeout, TimeoutSymbols};

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
    let mut dio1 = Input::new(p.PIN_16, Pull::Down);

    let mut config = Sx127xLoraConfig::default();
    config.frequency = LORA_FREQUENCY_HZ;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();

    sx127x.set_dio0::<RxDone>().await.unwrap();
    sx127x.set_dio1::<RxTimeout>().await.unwrap();

    spawner.spawn(heartbeat(Output::new(p.PIN_21, Level::Low)).unwrap());

    loop {
        sx127x.receive(Some(TimeoutSymbols::max())).await.unwrap();
        match select(dio0.wait_for_high(), dio1.wait_for_high()).await {
            Either::First(_) => {
                sx127x.clear_irq::<RxDone>().await.unwrap();
                match sx127x.read_rx_data().await {
                    Ok(buf) => {
                        let len: usize = buf.iter().filter(|c| **c != 0).count();
                        info!("rx buffer: {:a}", buf[..len])
                    },
                    Err(_) => error!("read_rx_data failed :(")
                }
            }
            Either::Second(_) => {
                info!("RxTimeout triggered");
                sx127x.clear_irq::<RxTimeout>().await.unwrap();
            }
        }
    }
}