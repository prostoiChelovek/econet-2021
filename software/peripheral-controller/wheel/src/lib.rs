#![no_std]

use rtt_target::rprintln;

use core::{ops::Add, fmt::Display};

use num_traits::{NumCast, Zero, Signed, ToPrimitive};

use pid::Pid;

use motor::{SetSpeed, GetSpeed};
use encoder::{Encoder, Update};

pub struct Wheel<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    speed: S,
    encoder: E,

    pid: Pid<f32>,
}

impl<S, E> Wheel<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    pub fn new(speed_controller: S, encoder: E, pid: Pid<f32>) -> Self {
        Self {
            speed: speed_controller,
            encoder,

            pid,
        }
    }
}

// TODO: it is kinda incorrect to use encoder's Update here, but it'll do for now
// TODO: clean up this fucking mess with traits
impl<S, E> Update for Wheel<S, E>
where
    S: SetSpeed + GetSpeed,
    <S as SetSpeed>::Speed: NumCast,
    <S as GetSpeed>::Speed: NumCast + Add + Display + Copy,
    <<S as GetSpeed>::Speed as Add>::Output: ToPrimitive + Display,
    E: Encoder
{
    fn update(&mut self, time_delta_seconds: f32) {
        self.encoder.update(time_delta_seconds);

        let velocity = -self.encoder.get_velocity();

        let control = self.pid.next_control_output(velocity).output;
        let control: <S as GetSpeed>::Speed = NumCast::from(control).unwrap();
        let new_speed = self.speed.get_speed() + control;

        rprintln!("target: {}, vel: {}, control: {}, new_speed: {}, speed_before: {}", self.pid.setpoint, velocity, control, new_speed, self.speed.get_speed());

        self.speed.set_speed(NumCast::from(new_speed).unwrap());
    }
}

impl<S, E> SetSpeed for Wheel<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    type Speed = f32;

    fn set_speed(&mut self, speed: Self::Speed) {
        self.pid.setpoint = speed;
    }
}

impl<S, E> GetSpeed for Wheel<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    type Speed = f32;

    fn get_speed(&mut self) -> Self::Speed {
        self.encoder.get_velocity()
    }
}
