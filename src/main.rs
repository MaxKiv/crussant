#![no_std]
#![no_main]

mod blink;
mod button;
mod epd;

use crate::{blink::blink_task, button::button_task};
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Io, Pin},
};

pub use esp_println::println as info;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();

    info!("initializing embassy");
    esp_hal_embassy::init(systimer.alarm0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio3;
    let button = io.pins.gpio9;

    info!("spawning tasks");
    spawner.spawn(blink_task(led.degrade())).unwrap();
    spawner.spawn(button_task(button.degrade())).unwrap();
}
