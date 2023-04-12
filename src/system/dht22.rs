use core::convert::Infallible;
use core::result::Result;
use dht_sensor::{dht22, DhtError, DhtReading};
use serde::Serialize;
use stm32f4xx_hal::gpio::{OpenDrain, Output, Pin};
use stm32f4xx_hal::timer::SysDelay;

type Dht22Pin = Pin<'B', 10, Output<OpenDrain>>;

#[derive(Serialize)]
pub struct Dht22Measurement {
    pub temp: f32,
    pub hum: f32,
}

pub struct Dht22Sensor {
    pin: Dht22Pin,
}

impl Dht22Sensor {
    pub fn new(pin: Dht22Pin) -> Self {
        let mut sensor = Dht22Sensor { pin };
        sensor.pin.set_high();
        sensor
    }

    pub fn read(&mut self, delay: &mut SysDelay) -> Result<Dht22Measurement, DhtError<Infallible>> {
        let result = dht22::Reading::read(delay, &mut self.pin)?;
        Ok(Dht22Measurement {
            hum: result.relative_humidity,
            temp: result.temperature,
        })
    }
}
