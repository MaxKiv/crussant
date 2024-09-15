#![no_std]
#![no_main]

mod blink;
mod button;
mod epd;

use crate::{blink::blink_task, button::button_task, epd::epd_task};
use embassy_executor::Spawner;
use epd_waveshare::prelude::WaveshareDisplay;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Io, Level, Pin},
};
use esp_println::println as info;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio3; // Green LED on my T8-C3
    let button = io.pins.gpio8; // Attached to button
                                // EPD pins
    info!("initializing embassy");
    esp_hal_embassy::init(systimer.alarm0);

    info!("spawning tasks");
    spawner.spawn(blink_task(led.degrade())).unwrap();
    spawner.spawn(button_task(button.degrade())).unwrap();

    // let sclk = io.pins.gpio6; // SPI clock pin
    // let miso = io.pins.gpio2; // Master In Slave Out pin
    // let mosi = io.pins.gpio7; // Master Out Slave In pi
    // let cs = io.pins.gpio10; // EPD chip select pin
    // let busy_in = io.pins.gpio9; // EPD busy pin
    // let dc = io.pins.gpio0; // EPD Data/Command pin
    // let rst = io.pins.gpio1; // EPD reset pin
    // spawner
    //     .spawn(epd_task(
    //         peripherals.SPI2,
    //         sclk.degrade(),
    //         mosi.degrade(),
    //         miso.degrade(),
    //         cs.degrade(),
    //         busy_in.degrade(),
    //         dc.degrade(),
    //         rst.degrade(),
    //     ))
    //     .unwrap();

    info!("Main task done");
}
