#![no_std]

use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use sx127xlora::types::{Bandwidth, CodingRate, HeaderMode, PreambleLength, SpreadingFactor, Sx127xLoraConfig};

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

// driver is full-duplex (NOT compiled with `half-duplex` feature) so 128 bytes i the max TX buffer size
pub const TX_PAYLOAD: [u8; 128] = [76, 111, 111, 107, 32, 97, 103, 97, 105, 110, 32, 97, 116, 32, 116, 104, 97, 116, 32, 100, 111, 116, 46, 32, 84, 104, 97, 116, 39, 115, 32, 104, 101, 114, 101, 46, 32, 84, 104, 97, 116, 39, 115, 32, 104, 111, 109, 101, 46, 32, 84, 104, 97, 116, 39, 115, 32, 117, 115, 46, 32, 79, 110, 32, 105, 116, 32, 101, 118, 101, 114, 121, 111, 110, 101, 32, 121, 111, 117, 32, 108, 111, 118, 101, 44, 32, 101, 118, 101, 114, 121, 111, 110, 101, 32, 121, 111, 117, 32, 107, 110, 111, 119, 44, 32, 101, 118, 101, 114, 121, 111, 110, 101, 32, 121, 111, 117, 32, 101, 118, 101, 114, 32, 104, 101, 97, 114, 100];


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
        true,
        Bandwidth::Bw125kHz,
        CodingRate::Cr4_7,
        LORA_FREQUENCY_HZ,
        HeaderMode::Explicit,
        PreambleLength::default(),
        SpreadingFactor::Sf11,
        0x12,
        true
    ).unwrap()
}

/// BW = 125_000 kHz
/// SF = 11
/// Rs = 61.03515625 (Rs = BW / 2 ** SF)
/// Ts = 16.384ms (Ts = (1 / Rs) * 1000)
/// HoppingPeriod: 400ms (FCC dwell time)
/// FreqHoppingPeriod (max) = 24.4ms (400ms / 16.384)
pub const FREQ_HOP_PERIOD_MS: u8 = 24;
pub fn fhss_config() -> Sx127xLoraConfig {
    Sx127xLoraConfig::new(
        false,
        Bandwidth::Bw125kHz,
        CodingRate::Cr4_7,
        LORA_FREQUENCY_HZ,
        HeaderMode::Explicit,
        PreambleLength::default(),
        SpreadingFactor::Sf11,
        0x12,
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