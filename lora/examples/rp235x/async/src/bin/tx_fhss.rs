//! TODO include all math
#![no_std]
#![no_main]

use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::{NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::{FHSS_CHANNELS, FHSS_CHANNELS_SIZE};
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{Bandwidth, CodingRate, FhssChangeChannel, PowerRamp, SpreadingFactor, TxConfig, TxDone};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut dio0 = Input::new(p.PIN_15, Pull::Down); // TxDone
    let mut dio1 = Input::new(p.PIN_16, Pull::Down); // FhssChangeChannel

    let mut config = Sx127xLoraConfig::default();
    config.bandwidth = Bandwidth::Bw250kHz;
    config.coding_rate = CodingRate::Cr4_5;
    config.frequency = FHSS_CHANNELS[0];
    config.spreading_factor = SpreadingFactor::Sf10;
    // preamble len: 8 (should be default)
    // symbol timeout: 5
    // nb symb hop: 4
    // iq inversion: false
    // crc enabled: true
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();
    sx127x.set_temp_monitor(false).await.unwrap();
    // symbol duration (~33ms) is > 16ms so enable low data rate optimization
    sx127x.set_low_data_rate_optimize(true).await.unwrap();
    sx127x.set_tx_config(TxConfig::new(14, PowerRamp::default(), false).unwrap()).await.unwrap();

    sx127x.set_dio0::<TxDone>().await.unwrap();
    sx127x.set_dio1::<FhssChangeChannel>().await.unwrap();
    sx127x.set_fhss_period(12).await.unwrap();

    // 128 bytes is the max in full duplex mode
    let payload = "pingpong".as_bytes();
    let mut hops_completed: usize = 0;
    let mut packets_sent: usize = 0;

    sx127x.transmit(&payload).await.unwrap();

    info!("entering main loop");
    loop {
        info!("hops completed: {}, packets sent: {}", hops_completed, packets_sent);
        info!("waiting...");
        match select(dio0.wait_for_high(), dio1.wait_for_high()).await {
            Either::First(_) => {
                sx127x.clear_irq::<TxDone>().await.unwrap();
                info!("TxDone triggered!");
                sx127x.set_frequency(FHSS_CHANNELS[0]).await.unwrap();
                packets_sent += 1;
                hops_completed = 0;
                // Timer::after_millis(1_000).await;
                // sx127x.transmit(&payload).await.unwrap();
            }
            Either::Second(_) => {
                //info!("FhssChangeChannel triggered!");
                let channel = sx127x.fhss_channel().await.unwrap();
                sx127x.set_frequency(FHSS_CHANNELS[channel as usize % FHSS_CHANNELS_SIZE]).await.unwrap();
                sx127x.clear_irq::<FhssChangeChannel>().await.unwrap();
                hops_completed += 1;
            }
        }
    }
}