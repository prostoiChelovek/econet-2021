#![no_std]

use embedded_hal::Qei;

use encoder::{Update, GetPosition, GetVelocity};

pub struct RotaryEncoder<QEI>
where
    QEI: Qei,
    QEI::Count: Into<i64>
{
    qei: QEI,

    pub ppr: f32,

    // angular
    position: f32,
    velocity: f32,

    last_count: i64,
}

impl<QEI> RotaryEncoder<QEI> 
where
    QEI: Qei,
    QEI::Count: Into<i64>
{
    pub fn new(qei: QEI, pulse_per_rev: f32) -> Self {
        Self {
            qei,

            ppr: pulse_per_rev,

            position: 0_f32,
            velocity: 0_f32,

            last_count: 0,
        }
    }
}

impl<QEI> Update for RotaryEncoder<QEI> 
where
    QEI: Qei,
    QEI::Count: Into<i64>
{
    fn update(&mut self, time_delta_seconds: f32) {
        let count: i64 = self.qei.count().into();
        let count_delta = (self.last_count - count) as i32;

        let last_position = self.position;
        self.position += count_delta as f32 / self.ppr;

        let position_delta = self.position - last_position;
        self.velocity = position_delta / time_delta_seconds;

        self.last_count = count;
    }
}

impl<QEI> GetPosition for RotaryEncoder<QEI> 
where
    QEI: Qei,
    QEI::Count: Into<i64>
{
    fn get_position(&self) -> f32 {
        self.position
    }
}

impl<QEI> GetVelocity for RotaryEncoder<QEI> 
where
    QEI: Qei,
    QEI::Count: Into<i64>
{
    fn get_velocity(&self) -> f32 {
        self.velocity
    }
}

