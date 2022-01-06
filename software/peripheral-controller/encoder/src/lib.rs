#![no_std]
#![feature(trait_alias)]

pub trait Update {
    fn update(&mut self, time_delta_seconds: f32);
}

pub trait GetPosition {
    fn get_position(&self) -> f32;
}

pub trait GetVelocity {
    fn get_velocity(&self) -> f32;
}

pub trait Encoder = Update + GetPosition + GetVelocity;

