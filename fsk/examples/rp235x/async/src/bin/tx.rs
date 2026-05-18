//! This async example shows how to use the FSK modem to transmit a packet and then respond to the
//! PacketSent interrupt on DIO0 once triggered.
#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use sx127xfsk::data_mode::PacketMode;
use sx127xfsk::dio::{PacketDio0Signal, PacketDio1Signal};
use sx127xfsk::driver::Sx127xFsk;
use sx127xfsk::types::PacketFormat;

const FREQUENCY_HZ: u32 = 915_000_000;

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
    let mut led = Output::new(p.PIN_20, Level::Low);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut dio0 = Input::new(p.PIN_15, Pull::Down);
    let mut dio1 = Input::new(p.PIN_14, Pull::Down);

    let mut fsk: Sx127xFsk<PacketMode, SpiDevice<'_, _, Spi<'_, _, _>, Output<'_>>> = Sx127xFsk::new(spi_dev).await.expect("driver init failed :(");
    fsk.set_frequency(FREQUENCY_HZ).await.expect("TODO");
    fsk.set_packet_format(PacketFormat::FixedLength).await.expect("TODO");
    fsk.set_payload_length(5).await.expect("TODO");
    fsk.set_dio0(PacketDio0Signal::PayloadReadyOrPacketSent).await.expect("TODO");
    fsk.set_dio1(PacketDio1Signal::FifoEmpty).await.expect("TODO");

    loop {
        fsk.transmit("howdy".as_bytes()).await.expect("TODO");
        match select(dio0.wait_for_high(), dio1.wait_for_high()).await {
            Either::First(_) => {
                info!("PacketSent triggered!");
                led.toggle();
                // PacketSent interrupt is automatically cleared when exiting Tx mode
                Timer::after(embassy_time::Duration::from_millis(3_000)).await;
            }
            Either::Second(_) => {
                info!("FifoEmpty triggered!");
            }
        }
    }
}