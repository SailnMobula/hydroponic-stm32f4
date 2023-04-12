use stm32f4xx_hal::hal::digital::v2::OutputPin;

pub struct Pump<E, P: OutputPin<Error = E>> {
    pin: P,
    is_pump_running: bool,
}

impl<E, P: OutputPin<Error = E>> Pump<E, P> {
    pub fn new(pin: P) -> Self {
        let mut pump = Self {
            pin,
            is_pump_running: false,
        };
        pump.turn_off().ok();
        pump
    }

    pub fn is_pump_running(&self) -> bool {
        self.is_pump_running
    }

    pub fn turn_on(&mut self) -> Result<(), E> {
        self.pin.set_high()?;
        self.is_pump_running = true;
        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<(), E> {
        self.pin.set_low()?;
        self.is_pump_running = false;
        Ok(())
    }
}
