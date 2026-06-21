#![no_std]

pub mod bits;
pub mod error;
pub mod registers;
pub mod spi;
pub mod calculate;

pub type Hz = u32;

pub const CHIP_VERSION: u8 = 0x12; // identifies silicon Version 1b
pub const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;
pub const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);
pub const FXOSC_HZ: u32 = 32_000_000;
pub const LF_MAX_HZ: u32 = 525_000_000;
pub const HF_MIN_HZ: u32 = 779_000_000;

pub enum Modem {
    Fsk = 0x0,
    LoRa = 0x1,
}