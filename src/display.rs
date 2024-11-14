use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Receiver};

use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::gpio::AnyPin;
use esp_hal::gpio::GpioPin;
use esp_hal::gpio::Input;
use esp_hal::gpio::Output;
use esp_hal::peripherals::SPI2;
use esp_hal::spi::master::SpiDma;
use esp_hal::spi::master::SpiDmaBus;
use esp_hal::spi::FullDuplexMode;
use esp_hal::Async;
use uom::si::pressure::hectopascal;
use uom::si::ratio::percent;
use uom::si::thermodynamic_temperature::degree_celsius;
use waveshare_154bv2::AsyncDisplay;
use waveshare_154bv2::Buffer;

use crate::dashboard::draw_dashboard;
use crate::error;
use crate::info;
use crate::sensor::SensorReading;

#[task]
pub async fn display_task(
    receiver: Receiver<'static, NoopRawMutex, SensorReading, 3>,
    spi_device: ExclusiveDevice<
        SpiDmaBus<'static, SPI2, FullDuplexMode, Async>,
        Output<'static, AnyPin>,
        Delay,
    >,
    busy: Input<'static, AnyPin>,
    rst: Output<'static, AnyPin>,
    dc: Output<'static, AnyPin>,
) {
    info!("Create display");
    let mut display = AsyncDisplay::new_with_individual_writes(spi_device, busy, rst, dc, Delay);

    info!("Initialize display");
    if let Err(error) = display.initialize().await {
        error!(" Cannot initialize display: {error:?}");
        return;
    }

    loop {
        info!("Wait for message from sensor");
        let sensor_reading = receiver.receive().await;

        if let Err(error) = report(sensor_reading).await {
            error!("Could not report sample: {error:?}");
        }
    }
}

async fn report(sensor_reading: SensorReading) -> Result<(), ReportError> {
    log_sample(&sensor_reading)?;

    update_display(&sensor_reading);

    Ok(())
}

fn update_display(
    sensor_reading: &(time::OffsetDateTime, crate::sensor::Sample),
) -> Result<(), ReportError> {
    let mut buffer = Buffer::new();

    info!("Draw dashboard on buffer");
    draw_dashboard(&mut buffer, now, sensor_reading).map_err(|_| ReportError::Display)?;
    info!("Draw buffer on display");
    display.draw_buffer(&buffer).await?;

    Ok(())
}

/// Print a sample to log
fn log_sample(reading: &SensorReading) -> Result<(), ReportError> {
    let (time, sample) = reading;

    let temperature = sample.temperature.get::<degree_celsius>();
    let humidity = sample.humidity.get::<percent>();
    let pressure = sample.pressure.get::<hectopascal>();

    info!("Received sample measured at {:?}", time);
    info!("┣ Temperature: {:.2} C", temperature);
    info!("┣ Humidity:    {:.2} %", humidity);
    info!("┗ Pressure:    {:.2} hPa", pressure);

    Ok(())
}

/// An error
#[derive(Debug)]
enum ReportError {
    /// An error occurred while logging the sample
    Log,
    /// An error occurred while refreshing the display
    Display,
}
