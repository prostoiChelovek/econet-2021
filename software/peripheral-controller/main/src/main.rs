#![no_main]
#![no_std]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init, set_print_channel};
use panic_probe as _;

use fugit::TimerDurationU32;

use stm32f4xx_hal::{
    prelude::*, delay, timer::{CounterUs, Timer}, pac, pac::TIM2,
    qei::Qei
};

use motor::{Motor, SetSpeed, RotationDirection};
use dc_motor::{TwoPinSetDirection, PwmSetSpeed};
use rotary_encoder::RotaryEncoder;

type MicrosTimer = CounterUs<TIM2>;
static MICROS_TIMER: Mutex<RefCell<Option<MicrosTimer>>> = Mutex::new(RefCell::new(None));

fn micros() -> u32 {
    static mut LOCAL_MICROS_TIMER: Option<MicrosTimer> = None;

    unsafe {
        let tim = LOCAL_MICROS_TIMER.get_or_insert_with(|| {
            cortex_m::interrupt::free(|cs| { 
                MICROS_TIMER.borrow(cs).replace(None).unwrap()
            })
        });
        
        tim.now().ticks()
    }
}

#[entry]
fn main() -> ! {
    let channels = 
        rtt_init! {
            up: {
                0: {
                    size: 1024
                    name: "Terminal"
                }
            }
            down: {
                0: {
                    size: 16
                    name: "Terminal"
                }
            }
        };
    set_print_channel(channels.up.0);

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let mut micros_timer = Timer::new(dp.TIM2, &clocks).counter_us();
    micros_timer.start(TimerDurationU32::from_ticks(u32::max_value())).unwrap();
    cortex_m::interrupt::free(|cs| {
        *MICROS_TIMER.borrow(cs).borrow_mut() = Some(micros_timer);
    });

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    let (in_1, in_2) = (gpiob.pb10.into_push_pull_output(), gpiob.pb4.into_push_pull_output());
    let en_pin = gpioa.pa8.into_alternate();
    let en_pwm = Timer::new(dp.TIM1, &clocks).pwm(en_pin, 2.khz());

    let directin = TwoPinSetDirection::new(in_1, in_2);
    let speed = PwmSetSpeed::new(en_pwm, 25);
    let mut motor = Motor::new(directin, speed);

    let encoder_pins = (gpioa.pa0.into_alternate(), gpioa.pa1.into_alternate());
    let encoder_timer = dp.TIM5;
    let qei = Qei::new(encoder_timer, encoder_pins);
    let mut encoder = RotaryEncoder::new(qei, 1440_f32);

    let mut delay = delay::Delay::new(cp.SYST, &clocks);

    loop {
        encoder.update(100);
        let pos = encoder.get_position();
        let vel = encoder.get_velocity();
        rprintln!("{} {}", pos, vel);

        delay.delay_ms(100_u32);
    }
}
