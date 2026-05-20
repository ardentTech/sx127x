#![no_std]

use embassy_rp::gpio::Output;
use embassy_time::Timer;

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