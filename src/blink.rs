use embassy_time::Timer;
use esp_hal::gpio::{ErasedPin, Level, Output};
use esp_println::println as info;

#[embassy_executor::task]
pub async fn blink_task(led: ErasedPin) {
    // configure pin as Output
    let mut led = Output::new(led, Level::High);

    loop {
        info!("blinking led...");
        blink_heartbeat(&mut led).await;
    }
}

/// toggles OutputPin in Heartbeat pattern
async fn blink_heartbeat(led: &mut Output<'_>) {
    Timer::after_millis(500).await;
    led.set_high();
    Timer::after_millis(100).await;
    led.set_low();

    Timer::after_millis(200).await;
    led.set_high();
    Timer::after_millis(100).await;
    led.set_low();
}
