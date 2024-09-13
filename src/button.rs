use embassy_time::{Duration, Timer};
use esp_hal::gpio::{ErasedPin, Input};
use esp_println::println as info;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(250);

#[embassy_executor::task]
pub async fn button_task(button: ErasedPin) {
    let mut button = Input::new(button, esp_hal::gpio::Pull::None);

    loop {
        debounce_buttonpress(&mut button);
        info!("button pressed");
    }
}

async fn debounce_buttonpress(button: &mut Input<'_>) {
    button.wait_for_high().await;
    Timer::after(DEBOUNCE_DURATION).await;
    button.wait_for_low().await;
}
