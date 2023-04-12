#![no_std]
#![no_main]

use core::cell::RefCell;
use core::convert::Infallible;
use core::fmt::Write;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use hydroponic_stm32f4::hydroponic::Hydroponic;
// Halt on panic
use hydroponic_stm32f4::system::{Dht22Sensor, Led, Pump};
use panic_halt as _;
use stm32f4xx_hal::gpio::{Alternate, Output, Pin};
use stm32f4xx_hal::pac::{TIM2, USART1};
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::timer::{CounterUs, Event, SysDelay};
use stm32f4xx_hal::{
    interrupt,
    pac::{Interrupt, Peripherals},
    prelude::*,
    rcc::RccExt,
    timer::SysTimerExt,
};

static G_COUNTER: AtomicU32 = AtomicU32::new(0);

static G_TIM: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM2() {
    G_COUNTER.fetch_add(1, Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut tim2) = G_TIM.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::Update);
        }
    });
}

#[entry]
fn main() -> ! {
    const _TOGGLE_TIME_MS: u32 = 1000;

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let (mut delay, serial, mut led, dht22, pump) = init_aux(cp, dp);
    let (mut serial_tx, _) = serial.split();

    let mut hydroponic = Hydroponic::new(dht22, pump);

    writeln!(serial_tx, "*** Starting Hydroponic Controller ***\r").unwrap();
    let mut current_time_s: u32 = 0;
    loop {
        let counter_s = G_COUNTER.load(Ordering::Relaxed);
        if current_time_s != counter_s {
            current_time_s = counter_s;
            match hydroponic.run(current_time_s, &mut delay) {
                Ok((interval, temp, runned_at, is_running)) => {
                    writeln!(serial_tx, "Hydroponic running, current time: {}s, watering interval: {}s temp: {}°C last runned: {}, is running: {}\r", current_time_s, interval, temp, runned_at, is_running).unwrap();
                    led.ok();
                }
                Err(e) => {
                    writeln!(serial_tx, "Error while running hydroponic: {:?}\r", e).unwrap();
                    led.err(&mut delay);
                }
            }
        }
    }
}

type SerialUart = Serial<USART1, (Pin<'A', 9, Alternate<7>>, Pin<'A', 10, Alternate<7>>), u8>;
type WateringPump = Pump<Infallible, Pin<'C', 14, Output>>;

fn init_aux(
    cp: cortex_m::Peripherals,
    dp: Peripherals,
) -> (SysDelay, SerialUart, Led, Dht22Sensor, WateringPump) {
    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.MHz())
        .sysclk(48.MHz())
        .require_pll48clk()
        .freeze();
    let delay = cp.SYST.delay(&clocks);

    let mut timer = dp.TIM2.counter(&clocks);
    timer.start(1.secs()).unwrap();
    timer.listen(Event::Update);

    cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();
    let gpiob = dp.GPIOB.split();

    let tx_pin = gpioa.pa9.into_alternate();
    let rx_pin = gpioa.pa10.into_alternate();

    let dht_pin = gpiob.pb10.into_open_drain_output();

    let led_pin = gpioc.pc13.into_push_pull_output();
    let pump_pin = gpioc.pc14.into_push_pull_output();

    let serial = dp
        .USART1
        .serial((tx_pin, rx_pin), 9600.bps(), &clocks)
        .unwrap();
    let led = Led::init(led_pin);
    let dht22 = Dht22Sensor::new(dht_pin);
    let pump = Pump::new(pump_pin);
    (delay, serial, led, dht22, pump)
}
