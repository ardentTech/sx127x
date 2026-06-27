//! This example shows how to use frequency hopping spread spectrum (FHSS) to transmit a packet. An external push button connected to GPIO 14 is used to
//! initiate transmission.
#![no_std]
#![no_main]

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::asm::wfi;
use cortex_m::delay::Delay;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::{PinState};
use embedded_hal_bus::spi::RefCellDevice;
use panic_probe as _;
use rp235x_hal::clocks::init_clocks_and_plls;
use rp235x_hal::{self as hal, pac, gpio};
use rp235x_hal::Clock;
use rp235x_hal::fugit::RateExtU32;
use rp235x_hal::gpio::{FunctionSioInput, FunctionSpi, Pin, PullDown};
use rp235x_hal::gpio::bank0::Gpio14;
use rp235x_hal::gpio::Interrupt::EdgeHigh;
use common::{fhss_config, pulse_led, Dio0, Dio1, GreenLed, FHSS_CHANNELS, FHSS_CHANNELS_SIZE, FREQ_HOP_PERIOD_MS, TX_PAYLOAD};
use sx127xlora::driver::Sx127xLora;
use sx127xlora::types::{FhssChangeChannel, PowerRamp, TxConfig, TxDone, OCP};
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use some_bsp;

type Btn = Pin<Gpio14, FunctionSioInput, PullDown>;
type Gpios = (Btn, Dio0, Dio1);
static GPIOS: Mutex<RefCell<Option<Gpios>>> = Mutex::new(RefCell::new(None));
static BTN_FLAG: AtomicBool = AtomicBool::new(false);
static DIO0_FLAG: AtomicBool = AtomicBool::new(false);
static DIO1_FLAG: AtomicBool = AtomicBool::new(false);

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[cortex_m_rt::entry] // this is available via rp235x_hal but rustrover fails to resolve it
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
    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = gpio::Pins::new(
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

    let btn = pins.gpio14.into_pull_down_input();
    btn.set_interrupt_enabled(EdgeHigh, true);
    let dio0: Dio0 = pins.gpio15.reconfigure();
    dio0.set_interrupt_enabled(EdgeHigh, true);
    let dio1: Dio1 = pins.gpio16.reconfigure();
    dio1.set_interrupt_enabled(EdgeHigh, true);

    let cs = pins.gpio13.into_push_pull_output_in_state(PinState::High);
    let spi_device = RefCellDevice::new(&spi_bus, cs, timer).unwrap();
    let mut sx127x = Sx127xLora::new_with_config(spi_device, fhss_config()).unwrap();
    sx127x.configure_tx(TxConfig::new(OCP::default(), 20, PowerRamp::default(), false).unwrap()).unwrap();
    sx127x.map_dio0::<TxDone>().unwrap();
    sx127x.map_dio1::<FhssChangeChannel>().unwrap();
    sx127x.set_hop_period(FREQ_HOP_PERIOD_MS).unwrap();

    let mut green_led: GreenLed = pins.gpio9.reconfigure();

    // Give away our pins by moving them into the `GLOBAL_PINS` variable.
    // We won't need to access them in the main thread again
    critical_section::with(|cs| {
        GPIOS.borrow(cs).replace(Some((btn, dio0, dio1)));
    });

    // Unmask the IRQ for I/O Bank 0 so that the RP2350's interrupt controller
    // (NVIC in Arm mode, or Xh3irq in RISC-V mode) will jump to the interrupt
    // function when the interrupt occurs. We do this last so that the interrupt
    // can't go off while it is in the middle of being configured
    unsafe {
        hal::arch::interrupt_unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    // Enable interrupts on this core
    unsafe {
        hal::arch::interrupt_enable();
    }
    info!("waiting for btn press");

    loop {
        wfi();
        if BTN_FLAG.load(Ordering::Relaxed) {
            info!("btn pressed!");
            sx127x.tx(&TX_PAYLOAD).unwrap();
            BTN_FLAG.store(false, Ordering::Relaxed);
        }
        if DIO0_FLAG.load(Ordering::Relaxed) {
            info!("TxDone");
            sx127x.clear_interrupt::<TxDone>().unwrap();
            sx127x.set_frequency(FHSS_CHANNELS[0]).unwrap();
            pulse_led(&mut green_led, &mut delay);
            DIO0_FLAG.store(false, Ordering::Relaxed);
        }
        if DIO1_FLAG.load(Ordering::Relaxed) {
            info!("FhssChangeChannel");
            let channel = sx127x.hop_channel().unwrap();
            sx127x.set_frequency(FHSS_CHANNELS[channel as usize % FHSS_CHANNELS_SIZE]).unwrap();
            sx127x.clear_interrupt::<FhssChangeChannel>().unwrap();
            debug!("hop to channel: {}", FHSS_CHANNELS[channel as usize % FHSS_CHANNELS_SIZE]);
            DIO1_FLAG.store(false, Ordering::Relaxed);
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
fn IO_IRQ_BANK0() {
    critical_section::with(|cs| {
        let mut maybe_gpios = GPIOS.borrow_ref_mut(cs);
        if let Some((btn, dio0, dio1)) = maybe_gpios.as_mut() {
            if btn.interrupt_status(EdgeHigh) {
                BTN_FLAG.store(true, Ordering::Relaxed);
                btn.clear_interrupt(EdgeHigh);
            }
            if dio0.interrupt_status(EdgeHigh) {
                DIO0_FLAG.store(true, Ordering::Relaxed);
                dio0.clear_interrupt(EdgeHigh);
            }
            if dio1.interrupt_status(EdgeHigh) {
                DIO1_FLAG.store(true, Ordering::Relaxed);
                dio1.clear_interrupt(EdgeHigh);
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
