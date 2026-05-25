#![no_std]

use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use sx127xlora::driver::Sx127xLoraConfig;
use sx127xlora::types::{Bandwidth, CodingRate, HeaderMode, SpreadingFactor};

pub const FHSS_CHANNELS_SIZE: usize = 13;
pub const FHSS_CHANNELS: [u32; FHSS_CHANNELS_SIZE] = [
    903_080_000,
    905_240_000,
    907_400_000,
    909_560_000,
    911_720_000,
    913_880_000,
    916_040_000,
    918_200_000,
    920_360_000,
    922_520_000,
    924_680_000,
    926_840_000,
    915_000_000,
];

pub const LORA_FREQUENCY_HZ: u32 = 915_000_000;

#[embassy_executor::task]
pub async fn heartbeat(mut pin: Output<'static>) {
    loop {
        pin.set_high();
        Timer::after_millis(250).await;
        pin.set_low();
        Timer::after_millis(750).await;
    }
}

pub fn debug_config() -> Sx127xLoraConfig {
    Sx127xLoraConfig::new(
        Bandwidth::Bw62_5kHz,
        CodingRate::Cr4_7,
        HeaderMode::Explicit,
        SpreadingFactor::Sf11,
        true
    ).unwrap()
}

pub enum Led {
    Green,
    Red
}

pub static PULSE_LED: Signal<CriticalSectionRawMutex, Led> = Signal::new();

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