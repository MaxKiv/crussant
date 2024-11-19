use bme280_rs::SensorMode;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embedded_hdc1080_rs::Hdc1080;
use esp_hal::i2c::I2c;
use esp_hal::peripherals::I2C0;
use esp_hal::rng::Rng;

use embassy_executor::task;

use embassy_sync::channel::Sender;

use embassy_time::Duration;
use embassy_time::Timer;

use esp_hal::Blocking;
use time::OffsetDateTime;

use uom::si::f32::Pressure;
use uom::si::f32::Ratio as Humidity;
use uom::si::f32::ThermodynamicTemperature as Temperature;

use crate::error;
use crate::info;

use crate::clock::Clock;

/// Interval to wait for sensor warmup
const WARMUP_INTERVAL: Duration = Duration::from_millis(10);
const WAIT_INTERVAL: Duration = Duration::from_secs(60);

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
    sender: Sender<'static, NoopRawMutex, SensorReading, 3>,
    // i2c: I2C0,
    i2c: esp_hal::i2c::I2c<'static, esp_hal::peripherals::I2C0, esp_hal::Blocking>,
    mut rng: Rng,
    clock: Clock,
) {
    info!("Initializing hdc1080 sensor");
    let mut hdc1080 = Hdc1080::new(i2c, Delay).unwrap();
    info!("Getting hdc1080 device id");
    let device_id = hdc1080.get_device_id().unwrap();
    info!("Getting hdc1080 manufacturing id");
    let manufacturing_id = hdc1080.get_man_id().unwrap();
    info!("hdc1080 device id: {device_id}");
    info!("hdc1080 manufacturing id: {manufacturing_id}");

    info!("Initializing ccs881 sensor");

    info!(
        "Waiting {}ms for configuration to be processed",
        WARMUP_INTERVAL.as_millis()
    );
    Timer::after(WARMUP_INTERVAL).await;

    loop {
        let hdc_reading = hdc1080
            .read()
            .map_err(|err| {
                error!("hdc1080 measurement error: {err:?}");
                SensorError::Sample
            })
            .unwrap();
        info!("hdc1080 reading: {hdc_reading:?}");

        let sensor_reading = sample(&mut rng, &clock).await.unwrap_or_else(|err| {
            error!("sensor measurement error: {err:?}");
            (OffsetDateTime::UNIX_EPOCH, Sample::random(&mut rng))
        });

        if let Err(send_err) = send(sensor_reading, &sender).await {
            error!("Sending measurement error: {send_err:?}");
        }

        info!("Wait {}s for next sample", WAIT_INTERVAL.as_secs());
        Timer::after(WAIT_INTERVAL).await;
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
