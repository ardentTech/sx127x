#![no_std]

pub mod bits;
pub mod error;
pub mod registers;
pub mod spi;
pub mod calculate;

// identifies silicon Version 1b
pub const CHIP_VERSION: u8 = 0x12;
pub const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;
pub const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);
pub const FXOSC_HZ: u32 = 32_000_000;

pub type Hz = u32;