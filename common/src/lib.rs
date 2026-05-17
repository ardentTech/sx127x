#![no_std]

pub mod bits;
pub mod error;
pub mod registers;
pub mod spi;
pub mod calculate;

pub const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);
pub const FXOSC_HZ: u32 = 32_000_000;
pub const DEFAULT_FREQUENCY_HZ: u32 = 434_000_000;