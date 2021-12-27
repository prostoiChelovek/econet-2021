#![no_main]
#![no_std]

use cortex_m_rt::entry;

use rtt_target::{rprintln, rtt_init, set_print_channel};
use panic_probe as _;

use stm32f4xx_hal::{
    prelude::*, delay, timer::Timer, pac
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

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    // Set up the LED. On the Mini-F4 it's connected to pin PC13.
    let gpioa = dp.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    let gpiob = dp.GPIOB.split();
    let scl = gpiob
        .pb8
        .into_alternate_af4_open_drain();

    let sda = gpiob
        .pb7
        .into_alternate_af4_open_drain();

    let mut i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks);
    let mut data = [0u8; 8];
    rprintln!("requesting....");

    i2c.write_read(0x05, &[0x08], &mut data).unwrap();
    rprintln!("{:?}", data);

    // Create a delay abstraction based on general-pupose 32-bit timer TIM5
    let mut delay = delay::Delay::new(cp.SYST, &clocks);

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

