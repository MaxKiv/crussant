use embassy_executor::task;

use embassy_time::Duration;
use embassy_time::Timer;

use esp_hal::gpio::AnyPin;
use esp_hal::gpio::Level;

use crate::info;
use crate::trace;

use Level::*;
const HEARTBEAT_PATTERN: [(Level, Duration); 4] = [
    (High, Duration::from_millis(100)),
    (Low, Duration::from_millis(100)),
    (High, Duration::from_millis(100)),
    (Low, Duration::from_millis(700)),
];

#[task]
pub async fn blink_task(led: AnyPin) {
    // configure pin as Output to drive LED
    let mut led = esp_hal::gpio::Output::new(led, esp_hal::gpio::Level::High);

    loop {
        info!("Blinking LED");
        for (level, duration_ms) in HEARTBEAT_PATTERN {
            led.set_level(level);
            Timer::after(duration_ms).await;
            led.toggle();
        }
    }
}
