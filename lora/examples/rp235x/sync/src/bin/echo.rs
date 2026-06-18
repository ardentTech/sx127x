//! This example will echo whatever packet payload it receives. The green led will pulse on
//! `RxDone` and the red led will pulse on `TxDone`.
#![no_std]
#![no_main]

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::asm::wfi;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::PinState;
use embedded_hal_bus::spi::RefCellDevice;
use panic_probe as _;
use rp235x_hal::clocks::init_clocks_and_plls;
use rp235x_hal::{self as hal, entry};
use rp235x_hal::{Clock, pac};
use rp235x_hal::fugit::RateExtU32;
use rp235x_hal::gpio::{FunctionSpi, Pin, PullNone};
use rp235x_hal::gpio::bank0::{Gpio7, Gpio9};
use rp235x_hal::gpio::Interrupt::EdgeHigh;
use common::{pulse_led, Dio0, GreenLed, RedLed};
use sx127xlora::driver::{Sx127xLora};
use sx127xlora::types::{RxDone, Sx127xLoraConfig, TxDone};
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use some_bsp;

static DIO0: Mutex<RefCell<Option<Dio0>>> = Mutex::new(RefCell::new(None));
static DIO0_FLAG: AtomicBool = AtomicBool::new(false);

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = cortex_m::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);


    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    let timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let miso = pins.gpio12.into_function::<FunctionSpi>();
    let mosi = pins.gpio11.into_function::<FunctionSpi>();
    let sck = pins.gpio10.into_function::<FunctionSpi>();
    let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI1, (mosi, miso, sck));
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16.MHz(),
        embedded_hal::spi::MODE_0,
    );
    let spi_bus = RefCell::new(spi);

    let dio0: Dio0 = pins.gpio15.reconfigure();
    dio0.set_interrupt_enabled(EdgeHigh, true);
    let mut green_led: GreenLed = pins.gpio9.reconfigure();
    let mut red_led: RedLed = pins.gpio7.reconfigure();

    let cs = pins.gpio13.into_push_pull_output_in_state(PinState::High);
    let spi_device = RefCellDevice::new(&spi_bus, cs, timer).unwrap();
    let mut config = Sx127xLoraConfig::default();
    config.use_crc = false;
    let mut sx127x = Sx127xLora::new(spi_device, config).unwrap();
    sx127x.map_dio0::<RxDone>().unwrap();

    loop {
        sx127x.map_dio0::<RxDone>().unwrap();
        sx127x.rx(None).unwrap();
        wfi();
        if DIO0_FLAG.load(Ordering::Relaxed) {
            sx127x.clear_interrupt::<RxDone>().unwrap();
            DIO0_FLAG.store(false, Ordering::Relaxed);
            match sx127x.rx_packet() {
                Ok(rxp) => {
                    let len: usize = rxp.payload.iter().filter(|c| **c != 0).count();
                    pulse_led(&mut green_led, &mut delay);

                    sx127x.map_dio0::<TxDone>().unwrap();
                    sx127x.tx(&rxp.payload).unwrap();
                    wfi();
                    if DIO0_FLAG.load(Ordering::Relaxed) {
                        sx127x.clear_interrupt::<TxDone>().unwrap();
                        pulse_led(&mut red_led, &mut delay);
                        DIO0_FLAG.store(false, Ordering::Relaxed);
                    }
                }
                Err(_) => error!("rx_packet() failed :(")
            }
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn IO_IRQ_BANK0() {
    critical_section::with(|cs| {
        let mut maybe_dio0 = DIO0.borrow_ref_mut(cs);
        if let Some(dio0) = maybe_dio0.as_mut() {
            if dio0.interrupt_status(EdgeHigh) {
                DIO0_FLAG.store(true, Ordering::Relaxed);
                dio0.clear_interrupt(EdgeHigh);
            }
        }
    })
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [rp235x_hal::binary_info::EntryAddr; 5] = [
    rp235x_hal::binary_info::rp_cargo_bin_name!(),
    rp235x_hal::binary_info::rp_cargo_version!(),
    rp235x_hal::binary_info::rp_program_description!(c"RP2350 Template"),
    rp235x_hal::binary_info::rp_cargo_homepage_url!(),
    rp235x_hal::binary_info::rp_program_build_attribute!(),
];

// End of file
