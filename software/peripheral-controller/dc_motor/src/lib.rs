#![no_std]

use num_traits::{Num, Zero};

use embedded_hal::{
    digital::v2::{OutputPin, PinState},
    PwmPin,
};

use motor::{RotationDirection, SetSpeed, SetDirection};

fn map_range<T: Num + Copy>(from_range: (T, T), to_range: (T, T), s: T) -> T {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

pub struct PwmSetSpeed<P>
where
    P: PwmPin,
    P::Duty: From<u8>
{
    pin: P,
    pub min_speed: u8
}

impl<P> PwmSetSpeed<P>
where
    P: PwmPin,
    P::Duty: From<u8>
{
    pub fn new(pin: P, min_speed: u8) -> Self {
        let mut pin = pin;
        pin.enable();

        Self { pin, min_speed }
    }
}

impl<P: PwmPin> SetSpeed for PwmSetSpeed<P> 
where
    P: PwmPin,
    P::Duty: From<u8> + Num + Ord + Copy
{
    type Speed = u8;

    fn set_speed(&mut self, speed: Self::Speed) {
        let zero_duty = P::Duty::zero();
        let hundred_duty = P::Duty::from(100);

        let speed = speed + self.min_speed;
        let speed = speed.max(0).min(100);
        let speed = P::Duty::from(speed);

        let duty = map_range(
            (zero_duty, hundred_duty),
            (zero_duty, self.pin.get_max_duty()),
            speed);
        self.pin.set_duty(duty);
    }
}

pub struct TwoPinSetDirection<F, B>
where
    F: OutputPin,
    B: OutputPin,
{
    fwd: F,
    back: B
}

impl<F, B> TwoPinSetDirection<F, B> 
where
    F: OutputPin,
    B: OutputPin,
{
    pub fn new(fwd_pin: F, back_pin: B) -> Self {
        TwoPinSetDirection {
            fwd: fwd_pin,
            back: back_pin,
        }
    }

    fn set_outs(&mut self, fwd: bool, back: bool) {
        self.fwd.set_state(PinState::from(fwd)).ok();
        self.back.set_state(PinState::from(back)).ok();
    }
}

impl<F, B> SetDirection for TwoPinSetDirection<F, B> 
where
    F: OutputPin,
    B: OutputPin,
{
    fn set_direction(&mut self, direction: RotationDirection) {
        match direction {
            RotationDirection::Clockwise => { self.set_outs(true, false) },
            RotationDirection::Counterclockwise => { self.set_outs(false, true) },
            RotationDirection::None => { self.set_outs(false, false) },
        }
    }
}

