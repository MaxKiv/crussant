#![feature(never_type)]
#![no_std]
#![no_main]

use blink::blink_task;
use clock::ClockError;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;
use embassy_time::Timer;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::dma::Dma;
use esp_hal::dma::DmaBufError;
use esp_hal::dma::DmaPriority;
use esp_hal::dma::DmaRxBuf;
use esp_hal::dma::DmaTxBuf;
use esp_hal::dma_buffers;
use esp_hal::gpio::Input;
use esp_hal::gpio::Io;
use esp_hal::gpio::Level;
use esp_hal::gpio::Output;
use esp_hal::gpio::Pin;
use esp_hal::gpio::Pull;
use esp_hal::i2c::I2c;
use esp_hal::peripherals::I2C0;
use esp_hal::peripherals::SPI2;
use esp_hal::rng::Rng;
use esp_hal::spi::master::Spi;
use esp_hal::spi::master::SpiDma;
use esp_hal::spi::master::SpiDmaBus;
use esp_hal::spi::FullDuplexMode;
use esp_hal::spi::SpiMode;
use esp_hal::Async;

use esp_hal_embassy::main;

use fugit::RateExtU32 as _;

use static_cell::StaticCell;

use core::convert::Infallible;

// use log::debug;
// use log::warn;
use log::error;
use log::info;
use log::trace;

use embassy_time::Duration;

use esp_hal::timer::timg::TimerGroup;

use esp_backtrace as _;
use esp_println as _;

// use defmt::info;

mod blink;

mod clock;
use clock::Clock;

mod display;
use display::display_task;

mod logger;

mod sensor;
use sensor::sensor_task;
use sensor::SensorReading;

mod dashboard;

/// Period to wait before going to deep sleep
const AWAKE_PERIOD: Duration = Duration::from_secs(3);

/// A channel between sensor sampler and display updater
static CHANNEL: StaticCell<Channel<NoopRawMutex, SensorReading, 3>> = StaticCell::new();

static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<I2C0, Async>>> = StaticCell::new();

/// Application entry point
/// Sets up logger and runs firmware
#[main]
async fn entry(spawner: Spawner) {
    logger::setup();
    info!("spawning main");
    match main(&spawner).await {
        Err(err) => {
            panic!("Main exited with {err:?}");
        }
        _ => {
            panic!("Main exited without error");
        }
    }
}

/// Fallible Main task
/// Spawns embassy tasks
async fn main(spawner: &Spawner) -> Result<!, Error> {
    info!("Initialize the HAL");
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    // get the IO driver
    info!("Initialize the IO driver");
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Green LED on my T8-C3 <3
    let led = io.pins.gpio3;

    info!("Initialize the RNG peripheral");
    let rng = Rng::new(peripherals.RNG);

    info!("Create Display and SPI Chip Select pins");
    let cs = Output::new(io.pins.gpio8, Level::Low);
    let busy = Input::new(io.pins.gpio9, Pull::Up);
    let rst = Output::new(io.pins.gpio10, Level::Low);
    // This is marked as uart RxD on my T8-C3
    let dc = Output::new(io.pins.gpio20, Level::Low);

    info!("Create SPI bus");
    let spi_bus = Spi::new(peripherals.SPI2, 25_u32.kHz(), SpiMode::Mode0)
        .with_sck(io.pins.gpio6)
        .with_mosi(io.pins.gpio7);

    info!("Initialize the DMA peripheral");
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    info!("Wrap SPI bus in a SPI DMA");
    let spi_dma: SpiDma<'_, SPI2, FullDuplexMode, Async> =
        spi_bus.with_dma(dma_channel.configure_for_async(false, DmaPriority::Priority0));

    info!("Initialize DMA buffers");
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).map_err(Error::DmaBufferCreation)?;
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).map_err(Error::DmaBufferCreation)?;

    info!("Create SPI DMA Bus");
    let spi_dma_bus = SpiDmaBus::new(spi_dma, dma_rx_buf, dma_tx_buf);
    let spi_device = ExclusiveDevice::new(spi_dma_bus, cs, Delay).map_err(|err| {
        error!("Error creating SPI ExclusiveDevice {err}");
        Error::SpiBusCreation
    })?;

    info!("Creating I2C pins");
    let sda = io.pins.gpio2;
    let scl = io.pins.gpio4;

    info!("Creating I2C device");
    let i2c_bus = I2C_BUS.init(Mutex::new(I2c::new_with_timeout_async(
        peripherals.I2C0,
        sda,
        scl,
        100.kHz(),
        Some(20),
    )));
    // let i2c = I2c::new_with_timeout(peripherals.I2C0, sda, scl, 400.kHz(), Some(20));

    info!("Creating Clock");
    let clock = Clock::new();
    info!("Now is {}", clock.now().map_err(Error::Clock)?);

    info!("Create channel");
    let channel: &'static mut _ = CHANNEL.init(Channel::new());
    let receiver = channel.receiver();
    let sender = channel.sender();

    info!("Initialising Embassy");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    info!(" --- Spawning tasks --- ");
    info!("Spawning blink task");
    spawner.must_spawn(blink_task(led.degrade()));
    info!("Spawning sensor task");
    spawner.must_spawn(sensor_task(sender, i2c_bus, rng, clock.clone()));
    info!("Spawning display task");
    spawner.must_spawn(display_task(receiver, spi_device, busy, rst, dc));

    // info!("Stay awake for {}s", AWAKE_PERIOD.as_secs());
    // Timer::after(AWAKE_PERIOD).await

    // info!("Go to sleep for {}s", DEEP_SLEEP_DURATION.as_secs());
    //
    // clock.save_to_rtc_memory(DEEP_SLEEP_DURATION);
    // enter_deep_sleep(peripherals.LPWR, DEEP_SLEEP_DURATION.into());
    //
    // info!("Awoken");

    loop {
        Timer::after(Duration::from_secs(1200)).await;
    }
}

/// An error
#[derive(Debug)]
enum Error {
    /// An impossible error existing only to satisfy the type system
    Impossible(Infallible),

    SpiBusCreation,

    DmaBufferCreation(DmaBufError),

    Clock(ClockError),
}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Self {
        Self::Impossible(error)
    }
}
