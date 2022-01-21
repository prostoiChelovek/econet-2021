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

            pub type _WheelT = Wheel<_motor::Motor, _encoder::Encoder>;

            pub use _motor::Motor as MotorT;
            pub use _encoder::Encoder as EncoderT;
        }
    };
}

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [USART1])]
mod app {
    use core::fmt::Write;

    use rtt_target::{rtt_init, set_print_channel, rprintln};

    use stm32f4xx_hal::{
        prelude::*,
        pac, pac::{TIM1, TIM3, TIM5, USART2, I2C1},
        gpio::{
            gpioa::{PA0, PA1, PA12},
            gpiob::{PB3, PB4, PB5, PB10, PB6, PB8, PB9},
            gpioc::{PC5, PC6, PC7, PC8, PC9},
            Output, PushPull, Alternate, OpenDrain
        },
        delay::Delay,
        timer::{monotonic::MonoTimer, Timer},
        pwm::{PwmChannel, C1, C2},
        qei::Qei,
        serial, serial::Serial, i2c, i2c::I2c
    };
    use shared_bus_rtic::SharedBus;

    use pid::Pid;
    use adxl343::{Adxl343, accelerometer::Accelerometer};

    use motor::{Motor, SetSpeed};
    use dc_motor::{TwoPinSetDirection, PwmSetSpeed};
    use encoder::*;
    use rotary_encoder::RotaryEncoder;
    use wheel::Wheel;
    use servo::Servo;
    use chassis::{chassis::{Chassis, ChassisPosition, ChassisSpeed}, movement_controller::{MovementController, MoveRelative}};
    use itg3205::Itg3205;
    use drawers_controller::Drawers;

    type OutPP = Output<PushPull>;

     #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimer<pac::TIM2, 1_000_000>;

    wheel_alias!(left_wheel, PB10, PB3, TIM1, C1, PB4, PB5, TIM3, 2_u8);
    wheel_alias!(right_wheel, PC7, PB6, TIM1, C2, PA0, PA1, TIM5, 2_u8);

    mod i2c_bus { 
        use super::*;

        type PinMode = Alternate<OpenDrain, 4_u8>;
        pub type I2cT = I2c<I2C1, (PB8<PinMode>, PB9<PinMode>)>;
        pub type BusT = SharedBus<I2cT>;

        pub fn create(i2c: I2cT) -> BusT {
            shared_bus_rtic::new!(i2c, I2cT)
        }
    }

    mod gy85 {
        use super::*;

        pub type AccelerometerT = Adxl343<i2c_bus::BusT>;
        pub type GyroT = Itg3205<i2c_bus::BusT>;

        pub struct Gy85(pub AccelerometerT, pub GyroT);
    }

    mod drawers {
        use super::*;

        type SetDirectionT = TwoPinSetDirection<PC9<OutPP>, PC8<OutPP>>;
        type EnablesT = (PC6<OutPP>, PC5<OutPP>, PA12<OutPP>);
        pub type DrawersT = Drawers<SetDirectionT, EnablesT>;
    }

    type SerialT = serial::Tx<USART2>;

    const WHEEL_RADIUS: f32 = 37.0;
    const WHEEL_MIN_SPEED_PERCENT: u8 = 25;
    const WHEEL_MAX_ROTARY_SPEED: f32 =  1.4;
    const WHEEL_ENCODER_PPR: f32 = 1440.0;

    const SERVO_MAX_DISTANCE: f32 = 20_00.0;
    const SERVO_MAX_TARRGET_DISTANCE: f32 = 1.0;

    const WHEELS_DISTANCE: f32 = 17.0;

    #[shared]
    struct Shared {
        chassis: MovementController<
            Chassis<
                Servo<left_wheel::MotorT, left_wheel::EncoderT>,
                Servo<right_wheel::MotorT, right_wheel::EncoderT>,
                gy85::GyroT
            >>,
        serial: SerialT,
    }

    #[local]
    struct Local {
        x: f32,
        y: f32
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
        let mut serial = Serial::tx(ctx.device.USART2, tx_pin, 115200.bps(), &clocks).unwrap();

        let mut delay = Delay::new(ctx.core.SYST, &clocks);

        let scl = gpiob
            .pb8
            .into_alternate()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiob
            .pb9
            .into_alternate()
            .internal_pull_up(true)
            .set_open_drain();
        let i2c = I2c::new(ctx.device.I2C1, (scl, sda), 400.khz(), &clocks);
        let bus = i2c_bus::create(i2c);

        let mut gyro = Itg3205::new(bus).unwrap();
        gyro.calibrate(100, &mut delay);

        let (left_wheel, right_wheel) = {
            let en_pins = (gpioa.pa8.into_alternate(), gpioa.pa9.into_alternate());
            let en_pwms = Timer::new(ctx.device.TIM1, &clocks).pwm(en_pins, 2.khz());
            let (left_en_pwm, right_en_pwm) = en_pwms;

            let speed_pid = Pid::new(0.25, 0.02, 1.0,
                               100.0, 100.0, 100.0,
                               100.0,
                               0.0);
            let position_pid = Pid::new(500.0, 0.001, 4000.0,
                               1.0, 0.1, 1.0,
                               1.0,
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

                let wheel = Wheel::new(motor, encoder, speed_pid.clone(), WHEEL_MAX_ROTARY_SPEED, WHEEL_RADIUS);

                Servo::new(wheel, position_pid, SERVO_MAX_DISTANCE, SERVO_MAX_TARRGET_DISTANCE)
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

                let wheel = Wheel::new(motor, encoder, speed_pid.clone(), WHEEL_MAX_ROTARY_SPEED, WHEEL_RADIUS);

                Servo::new(wheel, position_pid, SERVO_MAX_DISTANCE, SERVO_MAX_TARRGET_DISTANCE)
            })
        };

        let chassis = Chassis::new(left_wheel, right_wheel, gyro, WHEELS_DISTANCE);
        let mut chassis = MovementController::new(chassis);
        chassis.set_speed(ChassisSpeed { linear: 45.0, angular: 60.0 } );

        let mono = Timer::new(ctx.device.TIM2, &clocks).monotonic();

        updater::spawn().ok();
        position_updater::spawn().ok();
        printer::spawn().ok();

        (
            Shared {
                serial,
                chassis,
            },
            Local {
                x: 0.0, y: 0.0
            },
            init::Monotonics(mono),
        )
    }

    #[task(shared = [serial, chassis])]
    fn printer(cx: printer::Context){
        let serial = cx.shared.serial;
        let chassis = cx.shared.chassis;

        (serial, chassis).lock(|serial, chassis| {
            let position = chassis.get_position();
            rprintln!("{:?}", position);
            writeln!(serial, "{} {} {}", position.linear.0, position.linear.1, position.angular).unwrap();
        });

        printer::spawn_after(25.millis()).ok();
    }

    #[task(shared = [chassis], local = [x, y])]
    fn position_updater(mut cx: position_updater::Context) {
        let (x, y) = (cx.local.x, cx.local.y);
        let new_position = ChassisPosition {
            linear: (*x * 200.0, *y * 200.0),
            angular: 90.0
        };

        *x += 0.1; *y += 0.1;
        if *x >= 1.0 { *x = 0.0; }
        if *y >= 1.0 { *y = 0.0; }

        cx.shared.chassis.lock(|chassis| {
            chassis.move_relative(new_position);
        });

        position_updater::spawn_after(15_000.millis()).ok();
    }

    #[task(shared = [chassis])]
    fn updater(mut cx: updater::Context) {
        const TIME_DELTA_SECONDS: f32 = 0.025;

        cx.shared.chassis.lock(|chassis| {
            chassis.update(TIME_DELTA_SECONDS);
        });

        updater::spawn_after(25.millis()).ok();
    }
}
