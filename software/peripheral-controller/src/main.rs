#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init, set_print_channel};

use nucleo_f401re::{
    hal::{prelude::*, delay}, pac
};

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

    // Set up the LED. On the Mini-F4 it's connected to pin PC13.
    let gpioa = dp.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // Create a delay abstraction based on general-pupose 32-bit timer TIM5
    let mut delay = delay::Delay::new(cp.SYST, clocks);

    loop {
        // On for 1s, off for 1s.
        led.set_high();
        rprintln!("ON");
        delay.delay_ms(1_000_u32);
        led.set_low();
        rprintln!("OFF");
        delay.delay_us(1_000_000_u32);
    }
}

