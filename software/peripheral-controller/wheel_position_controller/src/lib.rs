#![no_std]

use core::{ops::Add, fmt::Display};

use num_traits::{NumCast, ToPrimitive, bounds::Bounded};
use pid::Pid;

use motor::{SetSpeed, GetSpeed};
use encoder::{Encoder, Update, GetPosition};
use wheel::Wheel;

pub trait SetPosition {
    fn set_position(&mut self, position: f32);
}

pub struct WheelPositionController <S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder,
{
    wheel: Wheel<S, E>,

    pid: Pid<f32>,

    max_position: f32,
    max_target_distance: f32,

    pub max_speed: f32,
}

impl<S, E> WheelPositionController<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    pub fn new(wheel: Wheel<S, E>, pid: Pid<f32>, max_position: f32, target_position_epsilon: f32) -> Self {
        assert_ne!(max_position, 0.0);

        Self {
            wheel,

            pid,

            max_position,
            max_target_distance: target_position_epsilon,

            max_speed: 1.0,
        }
    }

    pub fn get_target_position(&self) -> f32 {
        self.denormalize_position(self.pid.setpoint)
    }

    fn normalize_position(&self, pos: f32) -> f32 {
        pos / self.max_position
    }

    fn denormalize_position(&self, pos: f32) -> f32 {
        pos * self.max_position
    }

    fn normalize_speed(&self, speed: f32) -> f32 {
        speed / self.wheel.max_speed
    }

    fn denormalize_speed(&self, speed: f32) -> f32 {
        speed * self.wheel.max_speed
    }
}

impl<S, E> SetSpeed for WheelPositionController<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    type Speed = f32;

    fn set_speed(&mut self, speed: Self::Speed) {
        assert_ne!(speed, 0.0);

        self.max_speed = speed;
    }
}

impl<S, E> GetSpeed for WheelPositionController<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    type Speed = f32;

    fn get_speed(&mut self) -> Self::Speed {
        self.wheel.get_speed()
    }
}

impl<S, E> GetPosition for WheelPositionController<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    fn get_position(&self) -> f32 {
        self.wheel.get_position()
    }
}

impl<S, E> SetPosition for WheelPositionController<S, E>
where
    S: SetSpeed + GetSpeed,
    E: Encoder
{
    fn set_position(&mut self, position: f32) {
        self.pid.setpoint = self.normalize_position(position);
    }
}

impl<S, E> Update for WheelPositionController<S, E>
where
    S: SetSpeed + GetSpeed,
    <S as SetSpeed>::Speed: NumCast + Bounded,
    <S as GetSpeed>::Speed: NumCast + Add + Copy,
    <<S as GetSpeed>::Speed as Add>::Output: ToPrimitive + Display,
    E: Encoder
{
    fn update(&mut self, time_delta_seconds: f32) {
        self.wheel.update(time_delta_seconds);

        let position = self.get_position();
        let position = self.normalize_position(position);

        let current_speed = self.wheel.get_speed();
        let current_speed = self.normalize_speed(current_speed);

        let control = self.pid.next_control_output(position).output;
        let new_speed = current_speed + control;
        let new_speed = self.denormalize_speed(new_speed).min(self.max_speed).max(-self.max_speed);

        self.wheel.set_speed(new_speed);
    }
}

