#![no_main]
#![no_std]

use panic_probe as _;

macro_rules! wheel_alias {
    ($name:ident, $dir_1_pin:ident, $dir_2_pin:ident, $pwm_timer:ident, $pwm_chan:ident, $qei_pin_1:ident, $qei_pin_2:ident) => {
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

                type QeiT = Qei<TIM5, ($qei_pin_1<EncoderPinMode>, $qei_pin_2<EncoderPinMode>)>;
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
        pac, pac::{TIM1, TIM5, USART2},
        gpio::{
            gpioa::{PA0, PA1},
            gpiob::{PB4, PB10},
            Output, PushPull, Alternate
        },
        timer::{monotonic::MonoTimer, Timer},
        pwm::{PwmChannel, C1},
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
    type EncoderPinMode = Alternate<PushPull, 2_u8>;

     #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimer<pac::TIM2, 1_000_000>;

    wheel_alias!(left_wheel, PB10, PB4, TIM1, C1, PA0, PA1);

    type SerialT = serial::Tx<USART2>;

    #[shared]
    struct Shared {
        wheel: left_wheel::WheelT,
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

        let tx_pin = gpioa.pa2.into_alternate();
        let serial = Serial::tx(ctx.device.USART2, tx_pin, 115200.bps(), &clocks).unwrap();

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

        let pid = Pid::new(0.25, 0.02, 1.0, // 0.25, 0.01, 0.25
                           100.0, 100.0, 100.0,
                           100.0,
                           0.0);
        let wheel = Wheel::new(motor, encoder, pid, 1.4);

        let mono = Timer::new(ctx.device.TIM2, &clocks).monotonic();

        updater::spawn().ok();
        speed_updater::spawn().ok();
        printer::spawn().ok();

        (
            Shared {
                wheel,
                serial
            },
            Local {
                i: 0.0
            },
            init::Monotonics(mono),
        )
    }

    #[task(shared = [serial, wheel])]
    fn printer(cx: printer::Context){
        let serial = cx.shared.serial;
        let wheel = cx.shared.wheel;

        (serial, wheel).lock(|serial, wheel| {
            let target_speed = wheel.get_target_speed();
            let speed = wheel.get_speed();
            writeln!(serial, "{} {}", target_speed, speed).unwrap();
        });

        printer::spawn_after(25.millis()).ok();
    }

    #[task(shared = [wheel], local = [i])]
    fn speed_updater(mut cx: speed_updater::Context) {
        let i = cx.local.i;
        let new_speed = 1.4_f32 * (*i);
        *i += 0.1;
        if *i >= 1.0 { *i = 0.0; }

        cx.shared.wheel.lock(|wheel| {
            wheel.set_speed(new_speed);
        });

        speed_updater::spawn_after(2000.millis()).ok();
    }

    #[task(shared = [wheel])]
    fn updater(mut cx: updater::Context) {
        const TIME_DELTA_SECONDS: f32 = 0.025;

        cx.shared.wheel.lock(|wheel| {
            wheel.update(TIME_DELTA_SECONDS);
        });

        updater::spawn_after(25.millis()).ok();
    }
}
