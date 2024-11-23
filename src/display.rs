use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Receiver};

use embassy_time::Delay;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::gpio::AnyPin;
use esp_hal::gpio::Input;
use esp_hal::gpio::Output;
use esp_hal::peripherals::SPI2;
use esp_hal::spi::master::SpiDmaBus;
use esp_hal::spi::FullDuplexMode;
use esp_hal::Async;
use uom::si::pressure::hectopascal;
use uom::si::ratio::percent;
use uom::si::thermodynamic_temperature::degree_celsius;
use waveshare_154bv2_rs::AsyncDisplay as Display;
use waveshare_154bv2_rs::Buffer;
use waveshare_154bv2_rs::Error as DisplayError;

use crate::dashboard::draw_dashboard;
use crate::dashboard::DashboardError;
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
    let mut display = Display::new_with_individual_writes(spi_device, busy, rst, dc, Delay);

    info!("Initialize display");
    if let Err(error) = display.initialize().await {
        error!(" Cannot initialize display: {error:?}");
        return;
    }

    loop {
        info!("Wait for message from sensor");
        let sensor_reading = receiver.receive().await;

        if let Err(error) = report(&mut display, sensor_reading).await {
            error!("Could not report sample: {error:?}");
        }
    }
}

async fn report<SPI, BUSY, RST, DC, DELAY>(
    display: &mut Display<SPI, BUSY, RST, DC, DELAY>,
    sensor_reading: SensorReading,
) -> Result<(), ReportError>
where
    SPI: SpiDevice,
    BUSY: Wait,
    RST: OutputPin,
    DC: OutputPin,
    DELAY: DelayNs,
{
    log_sample(&sensor_reading)?;
    update_display(display, &sensor_reading).await?;
    Ok(())
}

async fn update_display<SPI, BUSY, RST, DC, DELAY>(
    display: &mut Display<SPI, BUSY, RST, DC, DELAY>,
    sensor_reading: &(time::OffsetDateTime, crate::sensor::Sample),
) -> Result<(), ReportError>
where
    SPI: SpiDevice,
    BUSY: Wait,
    RST: OutputPin,
    DC: OutputPin,
    DELAY: DelayNs,
{
    let mut buffer = Buffer::new();

    info!("Draw dashboard on buffer");
    draw_dashboard(&mut buffer, sensor_reading).map_err(ReportError::Dashboard)?;
    info!("Draw buffer on display");
    display
        .draw_buffer(&buffer)
        .await
        .map_err(ReportError::Display)?;

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
    // Log,
    /// An error occurred while constructing the dashboard buffer
    Dashboard(DashboardError),
    /// An error occurred while updating the display
    Display(DisplayError),
}
