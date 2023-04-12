use crate::system::{Dht22Measurement, Dht22Sensor, Pump};
use core::convert::Infallible;
use stm32f4xx_hal::gpio::{Output, Pin};
use stm32f4xx_hal::timer::SysDelay;

pub struct Hydroponic {
    temp_sensor: Dht22Sensor,
    pump: Pump<Infallible, Pin<'C', 14, Output>>,
    runned_last_time_s: u32,
    watering_interval_s: u32,
    current_measurement: Dht22Measurement,
}

impl Hydroponic {
    const WATERING_INTERVAL_DEFAULT_S: (f32, u32) = (5.0, 180 * 60);
    const WATERING_DURATION_S: u32 = 15;
    const WATERING_INTERVAL_S: [(f32, u32); 6] = [
        (5.0, 180 * 60),
        (10.0, 120 * 60),
        (15.0, 90 * 60),
        (20.0, 60 * 60),
        (25.0, 30 * 60),
        (30.0, 15 * 60),
    ];

    pub fn new(temp_sensor: Dht22Sensor, pump: Pump<Infallible, Pin<'C', 14, Output>>) -> Self {
        Hydroponic {
            temp_sensor,
            pump,
            runned_last_time_s: 0,
            watering_interval_s: 15 * 60,
            current_measurement: Dht22Measurement {
                hum: 0.0,
                temp: 0.0,
            },
        }
    }

    fn get_watering_interval(measurement: &Dht22Measurement) -> u32 {
        let (_, duration) = Self::WATERING_INTERVAL_S
            .iter()
            .take_while(|(temp, _)| measurement.temp > *temp)
            .last()
            .unwrap_or(&Self::WATERING_INTERVAL_DEFAULT_S);
        *duration
    }

    pub fn run(
        &mut self,
        current_time_s: u32,
        delay: &mut SysDelay,
    ) -> Result<(u32, f32, u32, bool), &str> {
        match self.temp_sensor.read(delay) {
            Ok(reading) => {
                self.current_measurement = reading;
                self.watering_interval_s = Self::get_watering_interval(&self.current_measurement);
            }
            Err(_e) => return Err("Failed to read sensor"),
        }

        if self.should_start_watering(current_time_s) {
            self.runned_last_time_s = current_time_s;
            if self.pump.turn_on().is_err() {
                return Err("Failed to turn on pump");
            }
        }

        if self.should_stop_watering(current_time_s) {
            let result = self.pump.turn_off();
            if result.is_err() {
                return Err("Failed to stop pump");
            }
        }

        Ok((
            self.watering_interval_s,
            self.current_measurement.temp,
            self.runned_last_time_s,
            self.pump.is_pump_running(),
        ))
    }

    fn should_start_watering(&mut self, current_time_s: u32) -> bool {
        (current_time_s - self.runned_last_time_s) > self.watering_interval_s
            && !self.pump.is_pump_running()
    }

    fn should_stop_watering(&mut self, current_time_s: u32) -> bool {
        (current_time_s - self.runned_last_time_s) >= Self::WATERING_DURATION_S
            && self.pump.is_pump_running()
    }
}

//TODO: Test not running yet
// #[cfg(test)]
// extern crate std;
// mod tests {
//     use crate::hydroponic::Hydroponic;
//     use crate::system::Dht22Measurement;
//
//     #[test]
//     fn get_watering_interval_returns_correct() {
//         let test_data = [
//             (Dht22Measurement { hum: 0.0, temp: 4.9 }, 180 * 60),
//             (Dht22Measurement { hum: 0.0, temp: 9.9 }, 180 * 60),
//             (Dht22Measurement { hum: 0.0, temp: 14.9 }, 120 * 60),
//             (Dht22Measurement { hum: 0.0, temp: 19.9 }, 90 * 60),
//             (Dht22Measurement { hum: 0.0, temp: 24.9 }, 60 * 60),
//             (Dht22Measurement { hum: 0.0, temp: 29.9 }, 30 * 60),
//             (Dht22Measurement { hum: 0.0, temp: 34.9 }, 15 * 60)];
//
//         for (temp, expected_result) in test_data {
//             let result = Hydroponic::get_watering_interval(measurement);
//             assert_eq!(result, expected_result);
//         }
//     }
// }
