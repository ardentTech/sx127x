//! This example demonstrates CAD and TX by checking for channel activity before transmitting a 256 byte payload. The green led on GPIO 21 will pulse on
//! success, or the red les on GPIO 22 will pulse on error.
#![no_std]
#![no_main]

use defmt::warn;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::{example_config, led_task, Led, PULSE_LED, TX_PAYLOAD};
use sx127xlora::driver::{Sx127xLora};
use sx127xlora::types::{CadDetected, CadDone, PowerRamp, TxConfig, TxDone, OCP};

const TX_DELAY_MS: u64 = 3_000;

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
    let mut dio3 = Input::new(p.PIN_18, Pull::Down);

    let mut sx127x = Sx127xLora::new_with_config(spi_dev, example_config()).await.unwrap();
    sx127x.configure_tx(TxConfig::new(OCP::default(), 20, PowerRamp::default(), false).unwrap()).await.unwrap();

    sx127x.map_dio0::<TxDone>().await.unwrap();
    sx127x.map_dio3::<CadDone>().await.unwrap();

    spawner.spawn(led_task(Output::new(p.PIN_21, Level::Low), Output::new(p.PIN_22, Level::Low)).unwrap());

    loop {
        sx127x.start_cad().await.unwrap();
        dio3.wait_for_high().await;

        if !sx127x.interrupt_flag::<CadDetected>().await.unwrap() {
            sx127x.tx(&TX_PAYLOAD).await.unwrap();

            dio0.wait_for_high().await;
            sx127x.clear_interrupt::<TxDone>().await.unwrap();

            PULSE_LED.signal(Led::Green);
        } else {
            warn!("CadDetected triggered so TX not attempted");
            sx127x.clear_interrupt::<CadDetected>().await.unwrap();
        }
        sx127x.clear_interrupt::<CadDone>().await.unwrap();
        Timer::after_millis(TX_DELAY_MS).await;
    }
}