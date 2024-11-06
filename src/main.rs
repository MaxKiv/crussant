// Copyright Claudio Mattera 2024.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main crate

#![no_std]
#![no_main]

use embassy_executor::task;
use embassy_executor::Spawner;
use esp_hal_embassy::main;

use core::convert::Infallible;

// use log::info;

// use embassy_executor::Spawner;

use embassy_time::Duration;
use embassy_time::Timer;

use esp_hal::timer::timg::TimerGroup;

use esp_backtrace as _;
use esp_println as _;

// use static_cell::StaticCell;

use defmt::info;

// mod logger;

/// Duration of deep sleep
const DEEP_SLEEP_DURATION: Duration = Duration::from_secs(300);

/// Period to wait before going to deep sleep
const AWAKE_PERIOD: Duration = Duration::from_secs(3);

/// Period to wait before going to deep sleep
const LOG_PERIOD: Duration = Duration::from_secs(1);

/// Timers
// static TIMERS: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();

#[task]
async fn alive_task() {
    loop {
        info!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

/// Main task
#[main]
async fn main(spawner: Spawner) {
    // logger::setup();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // let peripherals = Peripherals::take();

    info!("Initialising Embassy");
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // let rng = Rng::new(peripherals.RNG);

    info!("Spawn tasks");
    spawner.must_spawn(alive_task());

    info!("Stay awake for {}s", AWAKE_PERIOD.as_secs());
    Timer::after(AWAKE_PERIOD).await;

    info!("Hello from main");
    Timer::after(LOG_PERIOD).await;

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
