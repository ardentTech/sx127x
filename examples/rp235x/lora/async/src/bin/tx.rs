//! This async example shows how to use the LoRa modem to transmit a packet and then respond to the
//! TxDone interrupt on DIO0 once triggered.
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
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use sx127x::lora::driver::{Sx127x, Sx127xConfig};
use sx127x::lora::registers::OP_MODE;
use sx127x::lora::types::{Dio0Signal, Interrupt, SpreadingFactor};

const FREQUENCY_HZ: u32 = 915_000_000;

#[embassy_executor::main]
async fn main(_task_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);
    let mut led = Output::new(p.PIN_20, Level::Low);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut dio0 = Input::new(p.PIN_15, Pull::Down);

    let mut config = Sx127xConfig::default();
    config.frequency = FREQUENCY_HZ;
    config.spreading_factor = SpreadingFactor::Sf12;
    let mut sx127x = Sx127x::new(spi_dev, config).await.expect("driver init failed :(");
    sx127x.set_temp_monitor(false).await.expect("disable temp monitor failed :(");
    // symbol duration (~33ms) is > 16ms so enable low data rate optimize
    sx127x.set_low_data_rate_optimize(true).await.expect("set_low_data_rate_optimize failed :(");
    sx127x.set_pa_boost(20).await.expect("set_amplifier_boost failed :(");

    sx127x.set_dio0(Dio0Signal::TxDone).await.expect("set_dio0 failed :(");

    loop {
        sx127x.transmit("howdy".as_bytes()).await.expect("transmit failed :(");
        info!("waiting for TxDone...");
        dio0.wait_for_high().await;

        info!("TxDone triggered!");
        led.toggle();
        sx127x.clear_interrupt(Interrupt::TxDone).await.expect("clear interrupt TxDone failed :(");
        Timer::after(embassy_time::Duration::from_millis(2_000)).await;
    }
}