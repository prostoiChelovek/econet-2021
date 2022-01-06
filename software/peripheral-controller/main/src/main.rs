#![no_main]
#![no_std]

use panic_probe as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {
    use rtt_target::{rtt_init, set_print_channel, rprintln};

    use stm32f4xx_hal::{
        prelude::*,
        pac, pac::{TIM1, TIM5},
        gpio::{
            gpioa::{PA0, PA1},
            gpiob::{PB4, PB10},
            Output, PushPull, Alternate
        },
        timer::{monotonic::MonoTimer, Timer},
        pwm::{PwmChannel, C1},
        qei::Qei
    };

    use pid::Pid;

    use motor::{Motor, SetSpeed, GetSpeed};
    use dc_motor::{TwoPinSetDirection, PwmSetSpeed};
    use encoder::*;
    use rotary_encoder::RotaryEncoder;
    use wheel::Wheel;

    type OutPP = Output<PushPull>;
    type EncoderPinMode = Alternate<PushPull, 2_u8>;

     #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimer<pac::TIM2, 1_000_000>;

    mod left_wheel {
        use super::*;

        mod _motor {
            use super::*;

            type SetDirectionT = TwoPinSetDirection<PB10<OutPP>, PB4<OutPP>>;
            type SetSpeedT = PwmSetSpeed<PwmChannel<TIM1, C1>>;
            pub type Motor = motor::Motor<SetDirectionT, SetSpeedT>;
        }

        mod _encoder {
            use super::*;

            type QeiT = Qei<TIM5, (PA0<EncoderPinMode>, PA1<EncoderPinMode>)>;
            pub type Encoder = RotaryEncoder<QeiT>;
        }

        pub type WheelT = Wheel<_motor::Motor, _encoder::Encoder>;
    }

    #[shared]
    struct Shared {
        wheel: left_wheel::WheelT,
    }

    #[local]
    struct Local { }

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
        let encoder = RotaryEncoder::new(qei, 1440_f32);

        let pid = Pid::new(10.0, 10.0, 10.0,
                           100.0, 100.0, 100.0,
                           100.0,
                           0.0);
        let mut wheel = Wheel::new(motor, encoder, pid);

        wheel.set_speed(1.0);

        let mono = Timer::new(ctx.device.TIM2, &clocks).monotonic();

        updater::spawn().ok();

        (
            Shared {
                wheel
            },
            Local {
            },
            init::Monotonics(mono),
        )
    }

    #[task(shared = [wheel])]
    fn updater(mut cx: updater::Context) {
        const TIME_DELTA_SECONDS: f32 = 0.1;

        cx.shared.wheel.lock(|wheel| {
            wheel.update(TIME_DELTA_SECONDS);
        });

        updater::spawn_after(100.millis()).ok();
    }
}
