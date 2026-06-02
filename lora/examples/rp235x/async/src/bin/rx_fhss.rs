//! This example shows how to receive a packet using frequency hopping spread spectrum (FHSS).
#![no_std]
#![no_main]

use core::cell::RefCell;
use defmt::{debug, error, info, unwrap};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_rp::{bind_interrupts, interrupt};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, SPI1};
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex};
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use common::{fhss_config, led_task, Led, FHSS_CHANNELS, FHSS_CHANNELS_SIZE, FREQ_HOP_PERIOD_MS, PULSE_LED};
use sx127xlora::driver::Sx127xLora;
use sx127xlora::types::{FhssChangeChannel, PreambleLength, RxConfig, RxDone};

type Lora = Mutex<CriticalSectionRawMutex, RefCell<Sx127xLora<SpiDevice<'static, CriticalSectionRawMutex, Spi<'static, SPI1, Async>, Output<'static>>>>>;

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_MED: InterruptExecutor = InterruptExecutor::new();

static LORA: StaticCell<Lora> = StaticCell::new();

#[interrupt]
unsafe fn SWI_IRQ_1() {
    unsafe { EXECUTOR_HIGH.on_interrupt() }
}

#[interrupt]
unsafe fn SWI_IRQ_0() {
    unsafe { EXECUTOR_MED.on_interrupt() }
}

#[embassy_executor::task]
async fn rx_done_task(lora: &'static Lora, mut pin: Input<'static>) {
    loop {
        pin.wait_for_rising_edge().await;
        {
            let sx127x_unlocked = lora.lock().await;
            sx127x_unlocked.borrow_mut().clear_interrupt::<RxDone>().await.unwrap();
            match sx127x_unlocked.borrow_mut().rx_packet().await {
                Ok(rxp) => {
                    let len: usize = rxp.payload.iter().filter(|c| **c != 0).count();
                    info!("rx payload: {:a}", rxp.payload[..len]);
                    info!("rx coding rate: {}, rssi: {} dBm, snr: {} dB", rxp.coding_rate, rxp.rssi, rxp.snr);
                    PULSE_LED.signal(Led::Green);
                }
                Err(_) => error!("read_rx_data failed :(")
            }
        }
    }
}

#[embassy_executor::task]
async fn change_channel_task(lora: &'static Lora, mut pin: Input<'static>) {
    loop {
        pin.wait_for_rising_edge().await;
        {
            let sx127x_unlocked = lora.lock().await;
            let mut sx127x = sx127x_unlocked.borrow_mut();
            let channel = sx127x.hop_channel().await.unwrap();
            sx127x.set_frequency(FHSS_CHANNELS[channel as usize % FHSS_CHANNELS_SIZE]).await.unwrap();
            sx127x.clear_interrupt::<FhssChangeChannel>().await.unwrap();
            debug!("hop to channel: {}", FHSS_CHANNELS[channel as usize % FHSS_CHANNELS_SIZE]);
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, Config::default());
    static SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Spi<'static, SPI1, Async>>> = StaticCell::new();

    let cs = Output::new(p.PIN_13, Level::High);
    let spi_dev = SpiDevice::new(SPI_BUS.init(Mutex::new(spi)), cs);
    let mut config = fhss_config();
    config.frequency = FHSS_CHANNELS[0];
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();
    sx127x.config_rx(RxConfig::new(true, PreambleLength::default())).await.unwrap();
    sx127x.map_dio0::<RxDone>().await.unwrap();
    sx127x.map_dio1::<FhssChangeChannel>().await.unwrap();
    sx127x.set_hop_period(FREQ_HOP_PERIOD_MS).await.unwrap();

    let lora = LORA.init(Mutex::new(RefCell::new(sx127x)));

    // High-priority executor: SWI_IRQ_1, priority level 2
    interrupt::SWI_IRQ_1.set_priority(Priority::P2);
    let spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_1);
    spawner.spawn(unwrap!(rx_done_task(lora, Input::new(p.PIN_15, Pull::Down))));

    // Medium-priority executor: SWI_IRQ_0, priority level 3
    interrupt::SWI_IRQ_0.set_priority(Priority::P3);
    let spawner = EXECUTOR_MED.start(interrupt::SWI_IRQ_0);
    spawner.spawn(unwrap!(change_channel_task(lora, Input::new(p.PIN_16, Pull::Down))));
    spawner.spawn(led_task(Output::new(p.PIN_21, Level::Low), Output::new(p.PIN_22, Level::Low)).unwrap());

    // kick-start the whole process
    {
        let lora_unlocked = lora.lock().await;
        lora_unlocked.borrow_mut().rx(None).await.unwrap();
    }
}