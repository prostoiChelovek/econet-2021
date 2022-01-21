#![no_std]

pub trait GetRotation {
    fn get_rotation(&mut self) -> f32;
}
