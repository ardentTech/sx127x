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
use {defmt_rtt as _, panic_probe as _};
use common::LORA_FREQUENCY_HZ;
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{CadDetected, CadDone, DeviceMode, SpreadingFactor, TxDone};

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
    let mut dio3 = Input::new(p.PIN_18, Pull::Down);

    let mut config = Sx127xLoraConfig::default();
    config.frequency = LORA_FREQUENCY_HZ;
    config.spreading_factor = SpreadingFactor::Sf12;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();
    sx127x.set_temp_monitor(false).await.unwrap();
    // symbol duration (~33ms) is > 16ms so enable low data rate optimization
    sx127x.set_low_data_rate_optimize(true).await.unwrap();
    // TODO i'm not even sure this pin is mapped...
    //sx127x.set_power_amplifier(PowerAmplifier::new(20).unwrap()).await.unwrap();
    sx127x.set_tx_config(20, false).await.unwrap();

    sx127x.set_dio0::<TxDone>().await.unwrap();
    sx127x.set_dio3::<CadDone>().await.unwrap();

    spawner.spawn(led_task(Output::new(p.PIN_21, Level::Low)).unwrap());

    loop {
        sx127x.set_device_mode(DeviceMode::CAD).await.unwrap();
        dio3.wait_for_high().await;
        info!("CadDone triggered");

        if !sx127x.irq_flag::<CadDetected>().await.unwrap() {
            sx127x.transmit("howdy".as_bytes()).await.unwrap();

            dio0.wait_for_high().await;
            info!("TxDone triggered");
            sx127x.clear_irq::<TxDone>().await.unwrap();

            PULSE_LED.signal(());
        } else {
            info!("CadDetected triggered so TX not attempted");
            sx127x.clear_irq::<CadDetected>().await.unwrap();
        }
        sx127x.clear_irq::<CadDone>().await.unwrap();
        Timer::after_millis(TX_DELAY_MS).await;
    }
}