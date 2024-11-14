use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use esp_hal::rng::Rng;

use embassy_executor::task;

use embassy_sync::channel::Sender;

use embassy_time::Duration;
use embassy_time::Timer;

use time::OffsetDateTime;

use uom::si::f32::Pressure;
use uom::si::f32::Ratio as Humidity;
use uom::si::f32::ThermodynamicTemperature as Temperature;

use crate::error;
use crate::info;

use crate::clock::Clock;

/// Interval to wait for sensor warmup
const WARMUP_INTERVAL: Duration = Duration::from_millis(10);
const WAIT_INTERVAL: Duration = Duration::from_millis(500);

/// A sample
#[derive(Clone, Debug, Default)]
pub struct Sample {
    /// Temperature sample
    pub temperature: Temperature,
    // Humidity sample
    pub humidity: Humidity,
    /// Pressure sample
    pub pressure: Pressure,
}

impl Sample {
    // #[allow(clippy::cast_precision_loss)]
    pub fn random(rng: &mut Rng) -> Self {
        let temperature_seed = rng.random() as f32 / u32::MAX as f32;
        let humidity_seed = rng.random() as f32 / u32::MAX as f32;
        let pressure_seed = rng.random() as f32 / u32::MAX as f32;

        let temperature = temperature_seed * (30.0 - 15.0) + 15.0;
        let humidity = humidity_seed * (80.0 - 20.0) + 20.0;
        let pressure = pressure_seed * (1010.0 - 990.0) + 990.0;

        Self::from((
            uom::si::f32::ThermodynamicTemperature::new::<
                uom::si::thermodynamic_temperature::degree_celsius,
            >(temperature),
            uom::si::f32::Ratio::new::<uom::si::ratio::percent>(humidity),
            uom::si::f32::Pressure::new::<uom::si::pressure::hectopascal>(pressure),
        ))
    }
}

impl From<(Temperature, Humidity, Pressure)> for Sample {
    fn from((temperature, humidity, pressure): (Temperature, Humidity, Pressure)) -> Self {
        Self {
            temperature,
            humidity,
            pressure,
        }
    }
}

/// A sensor reading, i.e. a tuple (time, sample)
pub type SensorReading = (OffsetDateTime, Sample);

#[task]
pub async fn sensor_task(
    mut rng: Rng,
    clock: Clock,
    sender: Sender<'static, NoopRawMutex, SensorReading, 3>,
) {
    info!("Create");
    info!("Initializing sensor");
    info!(
        "Waiting {}ms for configuration to be processed",
        WARMUP_INTERVAL.as_millis()
    );
    Timer::after(WARMUP_INTERVAL).await;

    loop {
        let sensor_reading = sample(&mut rng, &clock).await.unwrap_or_else(|err| {
            error!("sensor measurement error: {err:?}");
            (OffsetDateTime::UNIX_EPOCH, Sample::random(&mut rng))
        });

        info!("Wait {}s for next sample", WAIT_INTERVAL.as_secs());
        Timer::after(WAIT_INTERVAL).await;

        if let Err(send_err) = send(sensor_reading, &sender).await {
            error!("Sending measurement error: {send_err:?}");
        }
    }
}

async fn sample(rng: &mut Rng, clock: &Clock) -> Result<SensorReading, SensorError> {
    let sample = Sample::random(rng);
    let now = clock.now().map_err(|_| SensorError::Sample)?;
    Ok((now, sample))
}

async fn send(
    sensor_reading: SensorReading,
    sender: &Sender<'static, NoopRawMutex, SensorReading, 3>,
) -> Result<(), SensorError> {
    sender.send(sensor_reading).await;
    Ok(())
}

/// Error within sensor sampling
#[derive(Debug)]
enum SensorError {
    /// Error sampling the sensor
    Sample,
    /// Error sending the sample
    #[allow(dead_code)]
    Send,
}
