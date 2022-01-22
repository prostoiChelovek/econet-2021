#![no_std]

use embedded_hal::digital::v2::OutputPin;

use motor::SetDirection;

pub trait DrawerEnableControl {
    fn disable(&mut self, id: u8);
    fn enable(&mut self, id: u8);
}

// TODO: automate it
impl <A, B, C> DrawerEnableControl for (A, B, C)
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin
{
    fn disable(&mut self, id: u8) {
        match id {
            0 => self.0.set_low().ok(),
            1 => self.1.set_low().ok(),
            2 => self.2.set_low().ok(),
            _ => panic!("Drawer index out of bounds")
        };
    }

    fn enable(&mut self, id: u8) {
        match id {
            0 => self.0.set_high().ok(),
            1 => self.1.set_high().ok(),
            2 => self.2.set_high().ok(),
            _ => panic!("Drawer index out of bounds")
        };
    }
}

pub struct Drawers<D, ENS>
where
    D: SetDirection,
    ENS: DrawerEnableControl
{
    direction: D,
    enables: ENS
}

impl<D, ENS> Drawers<D, ENS>
where
    D: SetDirection,
    ENS: DrawerEnableControl
{
    pub fn new(direction: D, enables: ENS) -> Self {
        Self {
            direction,
            enables
        }
    }
}
