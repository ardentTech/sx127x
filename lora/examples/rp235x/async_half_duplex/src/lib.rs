#![no_std]

use embassy_rp::gpio::Output;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use sx127xlora::types::{Bandwidth, CodingRate, HeaderMode, PreambleLength, SpreadingFactor, Sx127xLoraConfig};

pub const LORA_FREQUENCY_HZ: u32 = 915_000_000;

pub const TX_PAYLOAD: [u8; 255] = [76, 111, 111, 107, 32, 97, 103, 97, 105, 110, 32, 97, 116, 32, 116, 104, 97, 116, 32, 100, 111, 116, 46, 32, 84, 104, 97, 116, 39, 115, 32, 104, 101, 114, 101, 46, 32, 84, 104, 97, 116, 39, 115, 32, 104, 111, 109, 101, 46, 32, 84, 104, 97, 116, 39, 115, 32, 117, 115, 46, 32, 79, 110, 32, 105, 116, 32, 101, 118, 101, 114, 121, 111, 110, 101, 32, 121, 111, 117, 32, 108, 111, 118, 101, 44, 32, 101, 118, 101, 114, 121, 111, 110, 101, 32, 121, 111, 117, 32, 107, 110, 111, 119, 44, 32, 101, 118, 101, 114, 121, 111, 110, 101, 32, 121, 111, 117, 32, 101, 118, 101, 114, 32, 104, 101, 97, 114, 100, 32, 111, 102, 44, 32, 101, 118, 101, 114, 121, 32, 104, 117, 109, 97, 110, 32, 98, 101, 105, 110, 103, 32, 119, 104, 111, 32, 101, 118, 101, 114, 32, 119, 97, 115, 44, 32, 108, 105, 118, 101, 100, 32, 111, 117, 116, 32, 116, 104, 101, 105, 114, 32, 108, 105, 118, 101, 115, 46, 32, 84, 104, 101, 32, 97, 103, 103, 114, 101, 103, 97, 116, 101, 32, 111, 102, 32, 111, 117, 114, 32, 106, 111, 121, 32, 97, 110, 100, 32, 115, 117, 102, 102, 101, 114, 105, 110, 103, 44, 32, 116, 104, 111, 117, 115, 97, 110, 100, 115, 32, 111, 102, 32, 99, 111, 110, 102, 105, 100, 101, 110, 116, 32, 114, 101, 108, 105];

pub fn half_duplex_config() -> Sx127xLoraConfig {
    Sx127xLoraConfig::new(
        false,
        Bandwidth::Bw125kHz,
        CodingRate::Cr4_5,
        LORA_FREQUENCY_HZ,
        HeaderMode::Explicit,
        false,
        PreambleLength::default(),
        SpreadingFactor::Sf7,
        0x12,
        false
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