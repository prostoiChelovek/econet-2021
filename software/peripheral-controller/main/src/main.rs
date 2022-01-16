#![no_main]
#![no_std]

use panic_probe as _;

macro_rules! wheel_alias {
    ($name:ident, $dir_1_pin:ident, $dir_2_pin:ident, $pwm_timer:ident, $pwm_chan:ident, $qei_pin_1:ident, $qei_pin_2:ident, $qei_tim:ident, $qei_af:literal) => {
        mod $name {
            use super::*;

            mod _motor {
                use super::*;

                type SetDirectionT = TwoPinSetDirection<$dir_1_pin<OutPP>, $dir_2_pin<OutPP>>;
                type SetSpeedT = PwmSetSpeed<PwmChannel<$pwm_timer, $pwm_chan>>;
                pub type Motor = motor::Motor<SetDirectionT, SetSpeedT>;
            }

            mod _encoder {
                use super::*;

                type EncoderPinMode = Alternate<PushPull, $qei_af>;
                type QeiT = Qei<$qei_tim, ($qei_pin_1<EncoderPinMode>, $qei_pin_2<EncoderPinMode>)>;
                pub type Encoder = RotaryEncoder<QeiT>;
            }

            pub type WheelT = Wheel<_motor::Motor, _encoder::Encoder>;
        }
    };
}

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {
    use core::fmt::Write;

    use rtt_target::{rtt_init, set_print_channel, rprintln};

    use stm32f4xx_hal::{
        prelude::*,
        pac, pac::{TIM1, TIM3, TIM5, USART2},
        gpio::{
            gpioa::{PA0, PA1},
            gpiob::{PB3, PB4, PB5, PB10, PB6},
            gpioc::PC7,
            Output, PushPull, Alternate
        },
        timer::{monotonic::MonoTimer, Timer},
        pwm::{PwmChannel, C1, C2},
        qei::Qei,
        serial, serial::Serial
    };

    use pid::Pid;

    use motor::{Motor, SetSpeed, GetSpeed};
    use dc_motor::{TwoPinSetDirection, PwmSetSpeed};
    use encoder::*;
    use rotary_encoder::RotaryEncoder;
    use wheel::Wheel;

    type OutPP = Output<PushPull>;

     #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimer<pac::TIM2, 1_000_000>;

    wheel_alias!(left_wheel, PB10, PB3, TIM1, C1, PB4, PB5, TIM3, 2_u8);
    wheel_alias!(right_wheel, PC7, PB6, TIM1, C2, PA0, PA1, TIM5, 2_u8);

    type SerialT = serial::Tx<USART2>;

    const WHEEL_RADIUS: f32 = 37.0;
    const WHEEL_MIN_SPEED_PERCENT: u8 = 25;
    const WHEEL_MAX_ROTARY_SPEED: f32 =  1.4;
    const WHEEL_ENCODER_PPR: f32 = 1440_f32;

    #[shared]
    struct Shared {
        left_wheel: left_wheel::WheelT,
        right_wheel: right_wheel::WheelT,
        serial: SerialT
    }

    #[local]
    struct Local {
        i: f32
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
        let gpioc = ctx.device.GPIOC.split();

        let tx_pin = gpioa.pa2.into_alternate();
        let serial = Serial::tx(ctx.device.USART2, tx_pin, 115200.bps(), &clocks).unwrap();

        let (left_wheel, right_wheel) = {
            let en_pins = (gpioa.pa8.into_alternate(), gpioa.pa9.into_alternate());
            let en_pwms = Timer::new(ctx.device.TIM1, &clocks).pwm(en_pins, 2.khz());
            let (left_en_pwm, right_en_pwm) = en_pwms;

            let pid = Pid::new(0.25, 0.02, 1.0,
                               100.0, 100.0, 100.0,
                               100.0,
                               0.0);

            ({
                let (in_1, in_2) = (gpiob.pb10.into_push_pull_output(), gpiob.pb3.into_push_pull_output());

                let directin = TwoPinSetDirection::new(in_1, in_2);
                let speed = PwmSetSpeed::new(left_en_pwm, WHEEL_MIN_SPEED_PERCENT);
                let motor = Motor::new(directin, speed);

                let encoder_pins = (gpiob.pb4.into_alternate(), gpiob.pb5.into_alternate());
                let encoder_timer = ctx.device.TIM3;
                let qei = Qei::new(encoder_timer, encoder_pins);
                let encoder = RotaryEncoder::new(qei, WHEEL_ENCODER_PPR, true);

                Wheel::new(motor, encoder, pid.clone(), WHEEL_MAX_ROTARY_SPEED, WHEEL_RADIUS)
            },
            {
                let (in_1, in_2) = (gpioc.pc7.into_push_pull_output(), gpiob.pb6.into_push_pull_output());

                let directin = TwoPinSetDirection::new(in_1, in_2);
                let speed = PwmSetSpeed::new(right_en_pwm, WHEEL_MIN_SPEED_PERCENT);
                let motor = Motor::new(directin, speed);

                let encoder_pins = (gpioa.pa0.into_alternate(), gpioa.pa1.into_alternate());
                let encoder_timer = ctx.device.TIM5;
                let qei = Qei::new(encoder_timer, encoder_pins);
                let encoder = RotaryEncoder::new(qei, WHEEL_ENCODER_PPR, true);

                Wheel::new(motor, encoder, pid.clone(), WHEEL_MAX_ROTARY_SPEED, WHEEL_RADIUS)
            })
        };

        let mono = Timer::new(ctx.device.TIM2, &clocks).monotonic();

        updater::spawn().ok();
        speed_updater::spawn().ok();
        printer::spawn().ok();

        (
            Shared {
                left_wheel,
                right_wheel,
                serial
            },
            Local {
                i: 0.0
            },
            init::Monotonics(mono),
        )
    }

    #[task(shared = [serial, left_wheel, right_wheel])]
    fn printer(cx: printer::Context){
        let serial = cx.shared.serial;
        let left_wheel = cx.shared.left_wheel;
        let right_wheel = cx.shared.right_wheel;

        (serial, left_wheel, right_wheel).lock(|serial, left_wheel, right_wheel| {
            let target_speed = left_wheel.get_target_speed();
            let speed = (left_wheel.get_speed(), right_wheel.get_speed());
            rprintln!("{:?}", speed);
            writeln!(serial, "{} {} {}", target_speed, speed.0, speed.1).unwrap();
        });

        printer::spawn_after(25.millis()).ok();
    }

    #[task(shared = [left_wheel], local = [i])]
    fn speed_updater(mut cx: speed_updater::Context) {
        let i = cx.local.i;
        let new_speed = 1.4_f32 * WHEEL_RADIUS * (*i);
        *i += 0.1;
        if *i >= 1.0 { *i = 0.0; }

        cx.shared.left_wheel.lock(|left_wheel| {
            left_wheel.set_speed(new_speed);
        });

        speed_updater::spawn_after(2000.millis()).ok();
    }

    #[task(shared = [left_wheel, right_wheel])]
    fn updater(mut cx: updater::Context) {
        const TIME_DELTA_SECONDS: f32 = 0.025;

        cx.shared.left_wheel.lock(|left_wheel| {
            left_wheel.update(TIME_DELTA_SECONDS);
        });
        cx.shared.right_wheel.lock(|right_wheel| {
            right_wheel.update(TIME_DELTA_SECONDS);
        });

        updater::spawn_after(25.millis()).ok();
    }
}
