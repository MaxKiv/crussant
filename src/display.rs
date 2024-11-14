use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Receiver};

use uom::si::pressure::hectopascal;
use uom::si::ratio::percent;
use uom::si::thermodynamic_temperature::degree_celsius;

use crate::error;
use crate::info;
use crate::sensor::Sample;
use crate::sensor::SensorReading;

#[task]
pub async fn display_task(
    receiver: Receiver<'static, NoopRawMutex, crate::sensor::SensorReading, 3>,
) {
    info!("Create display");
    // let mut display = AsyncDisplay::new_with_individual_writes(spi_device, busy, rst, dc, Delay);

    info!("Initialize display");
    // if let Err(error) = display.initialize().await {
    //     error!(" Cannot initialize display: {error:?}");
    //     return;
    // }

    loop {
        info!("Wait for message from sensor");
        let sensor_reading = receiver.receive().await;

        if let Err(error) = report(sensor_reading).await {
            error!("Could not report sample: {error:?}");
        }
    }
}

async fn report(reading: SensorReading) -> Result<(), ReportError> {
    log_sample(&reading)?;

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
}
