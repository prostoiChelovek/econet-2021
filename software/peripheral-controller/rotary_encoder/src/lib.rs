#![no_std]

use embedded_hal::Qei;

pub struct RotaryEncoder<QEI>
where
    QEI: Qei,
    QEI::Count: TryInto<i64>
{
    qei: QEI,
    pub ppr: u32,
    count: i32,
    last_count: i64
}

impl<QEI> RotaryEncoder<QEI> 
where
    QEI: Qei,
    QEI::Count: TryInto<i64>
{
    pub fn new(qei: QEI, pulse_per_rev: u32) -> Self {
        Self {
            qei,
            ppr: pulse_per_rev,
            count: 0,
            last_count: 0
        }
    }

    pub fn update(&mut self) {
        let current: i64 = self.qei.count().try_into().unwrap_or(0);
        self.count += (self.last_count - current) as i32;
        self.last_count = current;
    }

    pub fn get_count(&self) -> i32 {
        self.count
    }

    pub fn get_revolutions(&self) -> f32 {
        (self.count as f32) / (self.ppr as f32)
    }
}

