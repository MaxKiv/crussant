#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod epd;
mod led;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Io, Pin},
};
use rtt_target::rtt_init_print;

use crate::led::blink_task;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();
    esp_hal_embassy::init(systimer.alarm0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio3;

    spawner.spawn(blink_task(led.degrade())).unwrap();
}
