#![no_std]

use compare::{Compare, natural};
use num_traits::{Zero, Signed};

use core::cmp::Ordering::{Less, Equal, Greater};

#[derive(Debug)]
pub enum RotationDirection {
    Clockwise,
    Counterclockwise,
    None
}

pub trait SetDirection {
    fn set_direction(&mut self, direction: RotationDirection);
}

pub trait SetSpeed {
    type Speed;

    fn set_speed(&mut self, speed: Self::Speed);
}

pub struct Motor<D, S>
where
    D: SetDirection,
    S: SetSpeed
{
    dir: D,
    speed: S
}

impl<D, S> Motor<D, S>
where
    D: SetDirection,
    S: SetSpeed,
{
    pub fn new(direction_controller: D, speed_controller: S) -> Self {
        Self { 
            dir: direction_controller,
            speed: speed_controller
        }
    }
}

impl<D, S> SetSpeed for Motor<D, S>
where
    D: SetDirection,
    S: SetSpeed,
    S::Speed: Zero + Signed + Ord + Compare<S::Speed>
{
    type Speed = S::Speed;

    fn set_speed(&mut self, speed: Self::Speed) {
        let cmp = natural();
        let direction = match cmp.compare(&speed, &Self::Speed::zero()) {
            Less => { RotationDirection::Counterclockwise },
            Equal => { RotationDirection::None },
            Greater => { RotationDirection::Clockwise }
        };
        self.dir.set_direction(direction);
        self.speed.set_speed(speed);
    }
}

