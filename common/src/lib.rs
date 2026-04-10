#![no_std]

pub mod bits;
pub mod error;
pub mod registers;
pub mod spi;

pub const FXOSC_HZ: u32 = 32_000_000;
pub const FSTEP: f32 = (FXOSC_HZ as f32) / (2u32.pow(19) as f32);