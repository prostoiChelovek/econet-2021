#![no_main]
#![no_std]

// use defmt_rtt as _;
use cortex_m_rt::entry;
// use panic_probe as _;
use panic_halt as _;

use nucleo_f401re::{
    hal::{prelude::*, delay},
    pac, Button, Led,
};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
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
            delay.delay_ms(1_000_u32);
            led.set_low();
            delay.delay_us(1_000_000_u32);
        }
    }

    loop {}
}
