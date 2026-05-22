//! This example checks for channel activity before transmitting a packet. The high spread factor (SF)
//! results in a low bit rate, so low data rate optimization is enabled and the power amplifier is set
//! to max.
#![no_std]
#![no_main]

use defmt::{info};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::LORA_FREQUENCY_HZ;
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{CadDetected, CadDone, DeviceMode, PowerRamp, SpreadingFactor, TxConfig, TxDone};

const TX_DELAY_MS: u64 = 3_000;

enum Led {
    Green,
    Red
}

static PULSE_LED: Signal<CriticalSectionRawMutex, Led> = Signal::new();

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
pub async fn led_task(mut green: Output<'static>, mut red: Output<'static>) {
    loop {
        let pin = match PULSE_LED.wait().await {
            Led::Green => &mut green,
            Led::Red => &mut red
        };
        pin.set_high();
        Timer::after(embassy_time::Duration::from_millis(250)).await;
        pin.set_low();
    }
}

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

    let mut config = Sx127xLoraConfig::default();
    config.spreading_factor = SpreadingFactor::Sf12;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();
    // TODO? sx127x.set_temp_monitor(false).await.unwrap();
    sx127x.set_frequency(LORA_FREQUENCY_HZ).await.unwrap();
    // symbol duration (~33ms) is > 16ms so enable low data rate optimization
    sx127x.optimize_for_low_data_rate(true).await.unwrap();
    sx127x.config_tx(TxConfig::new(20, PowerRamp::default(), false).unwrap()).await.unwrap();

    sx127x.map_dio0::<TxDone>().await.unwrap();
    sx127x.map_dio3::<CadDone>().await.unwrap();

    spawner.spawn(led_task(Output::new(p.PIN_21, Level::Low), Output::new(p.PIN_22, Level::Low)).unwrap());

    loop {
        sx127x.start_cad().await.unwrap();
        dio3.wait_for_high().await;
        info!("CadDone triggered");

        if !sx127x.interrupt_flag::<CadDetected>().await.unwrap() {
            sx127x.tx("howdy".as_bytes()).await.unwrap();

            dio0.wait_for_high().await;
            info!("TxDone triggered");
            sx127x.clear_interrupt::<TxDone>().await.unwrap();

            PULSE_LED.signal(Led::Green);
        } else {
            info!("CadDetected triggered so TX not attempted");
            sx127x.clear_interrupt::<CadDetected>().await.unwrap();
        }
        sx127x.clear_interrupt::<CadDone>().await.unwrap();
        Timer::after_millis(TX_DELAY_MS).await;
    }
}