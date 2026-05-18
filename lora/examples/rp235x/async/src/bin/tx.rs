//! This example shows how to use the LoRa modem to transmit a packet and then respond to the
//! TxDone interrupt on DIO0 once triggered. The high spread factor (SF) results in a low bit rate,
//! so there is no explicit timer delay in this example.
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
use {defmt_rtt as _, panic_probe as _};
use common::LORA_FREQUENCY_HZ;
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{PowerAmplifier, SpreadingFactor, TxDone};

const TX_DELAY_MS: u64 = 3_000;

static PULSE_LED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
pub async fn led_task(mut pin: Output<'static>) {
    loop {
        PULSE_LED.wait().await;
        pin.set_high();
        Timer::after(embassy_time::Duration::from_millis(250)).await;
        pin.set_low();
        Timer::after(embassy_time::Duration::from_millis(750)).await;
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

    let mut config = Sx127xLoraConfig::default();
    config.frequency = LORA_FREQUENCY_HZ;
    config.spreading_factor = SpreadingFactor::Sf12;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();
    sx127x.set_temp_monitor(false).await.unwrap();
    // symbol duration (~33ms) is > 16ms so enable low data rate optimize
    sx127x.set_low_data_rate_optimize(true).await.unwrap();
    sx127x.set_power_amplifier(PowerAmplifier::new(20).unwrap()).await.unwrap();

    sx127x.set_dio0::<TxDone>().await.unwrap();
    spawner.spawn(led_task(Output::new(p.PIN_21, Level::Low)).unwrap());

    loop {
        sx127x.transmit("howdy".as_bytes()).await.unwrap();

        info!("waiting for TxDone...");
        dio0.wait_for_high().await;
        info!("TxDone triggered!");

        sx127x.clear_irq::<TxDone>().await.unwrap();
        PULSE_LED.signal(());
        Timer::after_millis(TX_DELAY_MS).await;
    }
}