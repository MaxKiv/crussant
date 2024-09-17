#![no_std]
#![no_main]

// mod tasks;

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("initializing embassy");
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(1000).await;

        info!("low");
        led.set_low();
        Timer::after_millis(1000).await;
    }

    //
    // let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    // esp_hal_embassy::init(systimer.alarm0);
    //
    // info!("spawning tasks");
    // spawner.spawn(blink_task(led.degrade())).unwrap();
    //
    // let button = io.pins.gpio8; // Attached to button
    // spawner.spawn(button_task(button.degrade())).unwrap();
    //
    // spawner.spawn(spi_task(spi)).unwrap();
    // // let mut adc1_config = AdcConfig::new();
    // // let adc_pin = adc1_config.enable_pin(
    // //     io.pins.gpio0,
    // //     esp_hal::analog::adc::Attenuation::Attenuation11dB,
    // // );
    // // let adc1 = Adc::new(peripherals.ADC1, adc1_config);
    // // spawner.spawn(adc_task(adc1, adc_pin)).unwrap();
    //
    // // let sclk = io.pins.gpio6; // SPI clock pin
    // // let miso = io.pins.gpio2; // Master In Slave Out pin
    // // let mosi = io.pins.gpio7; // Master Out Slave In pi
    // // let cs = io.pins.gpio10; // EPD chip select pin
    // // let busy_in = io.pins.gpio9; // EPD busy pin
    // // let dc = io.pins.gpio0; // EPD Data/Command pin
    // // let rst = io.pins.gpio1; // EPD reset pin
    // // spawner
    // //     .spawn(epd_task(
    // //         peripherals.SPI2,
    // //         sclk.degrade(),
    // //         mosi.degrade(),
    // //         miso.degrade(),
    // //         cs.degrade(),
    // //         busy_in.degrade(),
    // //         dc.degrade(),
    // //         rst.degrade(),
    // //     ))
    // //     .unwrap();
    //
    // info!("Main task done");
}
