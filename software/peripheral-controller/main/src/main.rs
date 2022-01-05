#![no_main]
#![no_std]

use panic_probe as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {
    use rtt_target::{rtt_init, set_print_channel};

    use stm32f4xx_hal::{
        prelude::*,
        pac, pac::TIM1,
        gpio::{Output, Pin, PushPull},
        timer::{monotonic::MonoTimer, Timer},
        pwm::{PwmChannel, C1},
        qei::Qei
    };

    use motor::{Motor, SetSpeed};
    use dc_motor::{TwoPinSetDirection, PwmSetSpeed};
    use rotary_encoder::RotaryEncoder;

     #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimer<pac::TIM2, 1_000_000>;

    #[shared]
    struct Shared { }

    // this should be illegal
    type MyMotor = Motor<TwoPinSetDirection<Pin<Output<PushPull>, 'B', 10_u8>, Pin<Output<PushPull>, 'B', 4_u8>>, PwmSetSpeed<PwmChannel<TIM1, C1>>>;
    #[local]
    struct Local {
        motor: MyMotor
    }

     #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
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

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

        let gpioa = ctx.device.GPIOA.split();
        let gpiob = ctx.device.GPIOB.split();

        let (in_1, in_2) = (gpiob.pb10.into_push_pull_output(), gpiob.pb4.into_push_pull_output());
        let en_pin = gpioa.pa8.into_alternate();
        let en_pwm = Timer::new(ctx.device.TIM1, &clocks).pwm(en_pin, 2.khz());

        let directin = TwoPinSetDirection::new(in_1, in_2);
        let speed = PwmSetSpeed::new(en_pwm, 25);
        let motor = Motor::new(directin, speed);

        let encoder_pins = (gpioa.pa0.into_alternate(), gpioa.pa1.into_alternate());
        let encoder_timer = ctx.device.TIM5;
        let qei = Qei::new(encoder_timer, encoder_pins);
        let mut _encoder = RotaryEncoder::new(qei, 1440_f32);

        let mono = Timer::new(ctx.device.TIM2, &clocks).monotonic();

        task1::spawn().ok();

        (
            Shared { },
            Local {
                motor
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [motor])]
    fn task1(cx: task1::Context) {
        cx.local.motor.set_speed(100);
    }
}
