// Copyright Claudio Mattera 2024.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main crate

#![no_std]
#![no_main]

use blink::blink_task;
use embassy_executor::task;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Io;
use esp_hal::gpio::Pin;
use esp_hal::rng::Rng;
use esp_hal_embassy::main;
use static_cell::StaticCell;

use core::convert::Infallible;

use log::error;
use log::info;
use log::trace;

// use embassy_executor::Spawner;

use embassy_time::Duration;
use embassy_time::Timer;

use esp_hal::timer::timg::TimerGroup;

use esp_backtrace as _;
use esp_println as _;

// use static_cell::StaticCell;

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

/// Period to wait before going to deep sleep
const AWAKE_PERIOD: Duration = Duration::from_secs(3);

/// Period to wait before going to deep sleep
const LOG_PERIOD: Duration = Duration::from_secs(1);

/// A channel between sensor sampler and display updater
static CHANNEL: StaticCell<Channel<NoopRawMutex, SensorReading, 3>> = StaticCell::new();

/// Timers
// static TIMERS: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();

/// Main task
#[main]
async fn main(spawner: Spawner) {
    logger::setup();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio3; // Green LED on my T8-C3

    info!("Initialising Embassy");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let rng = Rng::new(peripherals.RNG);

    info!("Creating Clock");
    let clock = Clock::new();
    info!("Now is {}", clock.now().unwrap());

    info!("Create channel");
    let channel: &'static mut _ = CHANNEL.init(Channel::new());
    let receiver = channel.receiver();
    let sender = channel.sender();

    info!("Spawning tasks");
    info!("Spawning blink task");
    spawner.must_spawn(blink_task(led.degrade()));
    info!("Spawning sensor task");
    spawner.must_spawn(sensor_task(rng, clock.clone(), sender));
    info!("Spawning display task");
    spawner.must_spawn(display_task(receiver));

    // info!("Stay awake for {}s", AWAKE_PERIOD.as_secs());
    // Timer::after(AWAKE_PERIOD).await;

    // info!("Go to sleep for {}s", DEEP_SLEEP_DURATION.as_secs());
    //
    // clock.save_to_rtc_memory(DEEP_SLEEP_DURATION);
    // enter_deep_sleep(peripherals.LPWR, DEEP_SLEEP_DURATION.into());
    //
    // info!("Awoken");
}

/// An error
#[derive(Debug)]
enum Error {
    /// An impossible error existing only to satisfy the type system
    Impossible(Infallible),
}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Self {
        Self::Impossible(error)
    }
}
