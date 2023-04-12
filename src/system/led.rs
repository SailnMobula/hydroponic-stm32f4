use stm32f4xx_hal::gpio::{Output, Pin};
use stm32f4xx_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use stm32f4xx_hal::timer::SysDelay;

type LedPin = Pin<'C', 13, Output>;

pub struct Led {
    pin: LedPin,
}

impl Led {
    pub fn init(pin: LedPin) -> Self {
        Led { pin }
    }

    pub fn on(&mut self) {
        self.pin.set_low();
    }

    pub fn off(&mut self) {
        self.pin.set_high();
    }

    pub fn blink(&mut self, delay: &mut SysDelay, ms: u32) {
        self.on();
        delay.delay_ms(ms);
        self.off();
        delay.delay_ms(ms);
    }

    pub fn ok(&mut self) {
        self.on();
    }

    pub fn err(&mut self, delay: &mut SysDelay) {
        for _ in 0..5 {
            self.blink(delay, 50);
        }
        self.off()
    }
}
