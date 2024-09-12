use embassy_time::Timer;
use esp_hal::gpio::{ErasedPin, Level, Output};
use esp_println::println;
use rtt_target::rprintln;

#[embassy_executor::task]
pub async fn blink_task(led: ErasedPin) {
    // configure pin as Output
    let mut led = Output::new(led, Level::High);

    loop {
        Timer::after_secs(1).await;
        println!("blinking led...");
        rprintln!("blinking led...");

        led.toggle();
    }
}
