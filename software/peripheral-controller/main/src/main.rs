#![no_main]
#![no_std]

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init, set_print_channel};
use panic_probe as _;

use stm32f4xx_hal::{
    prelude::*, delay, timer::Timer, pac
};

use motor::{Motor, SetSpeed};
use dc_motor::{TwoPinSetDirection, PwmSetSpeed};

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

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    let (in_1, in_2) = (gpiob.pb10.into_push_pull_output(), gpiob.pb4.into_push_pull_output());
    let en_pin = gpioa.pa8.into_alternate();
    let en_pwm = Timer::new(dp.TIM1, &clocks).pwm(en_pin, 2.khz());

    let directin = TwoPinSetDirection::new(in_1, in_2);
    let speed = PwmSetSpeed::new(en_pwm, 25);
    let mut motor = Motor::new(directin, speed);

    let mut delay = delay::Delay::new(cp.SYST, &clocks);

    loop {
        for i in -100_i8..100_i8 {
            motor.set_speed(i);
            delay.delay_ms(100u8);
        }
    }
}

