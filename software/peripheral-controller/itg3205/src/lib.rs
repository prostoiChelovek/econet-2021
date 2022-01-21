#![no_std]

// logic copied from https://github.com/ikiselev/ITG3205/

mod register;

use crate::register::Register;

use core::fmt::Debug;

use embedded_hal as hal;
use hal::blocking::{i2c::{Write, WriteRead}, delay::DelayMs};

use micromath::vector::{F32x3, I16x3};

pub const ADDRESS: u8 = 0x68;

const LSB_DEG: f32 = 14.375; // TODO: figure out what is this

pub struct Itg3205<I2C> {
    i2c: I2C,

    offset: F32x3
}

impl<I2C, E> Itg3205<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
    E: Debug,
{
    pub fn new(i2c: I2C) -> Result<Self, E> {
        let mut itg3205 = Self {
            i2c,

            offset: F32x3::new(0.0, 0.0, 0.0)
        };

        itg3205.write_register(Register::PWR_MGM, 0x00)?;
        itg3205.write_register(Register::SMPLRT_DIV, 0x07)?; // Fsample = 1kHz / (7 + 1) = 125Hz, or 8ms per sample
        itg3205.write_register(Register::DLPF_FS, 0x1B)?; // +/- 2000 dgrs/sec, 1KHz, Low Pass Filter Bandwidth: 98Hzh
        itg3205.write_register(Register::INT_CFG, 0x00)?;

        Ok(itg3205)
    }

    pub fn calibrate<D: DelayMs<u8>>(&mut self, num_samples: u16, delay: &mut D) -> Result<(), E> {
        let mut accum = F32x3::new(0.0, 0.0, 0.0);

        for i in 0..num_samples {
            let val: F32x3 = self.read_raw()?.into();
            accum.x += val.x;
            accum.y += val.y;
            accum.z += val.z;

            delay.delay_ms(5);
        }

        self.offset.x = -(accum.x / num_samples as f32);
        self.offset.y = -(accum.y / num_samples as f32);
        self.offset.z = -(accum.z / num_samples as f32);

        Ok(())
    }

    pub fn read_raw(&mut self) -> Result<I16x3, E> {
        let x = self.read_register(Register::GYRO_XOUT_H)?;
        let y = self.read_register(Register::GYRO_YOUT_H)?;
        let z = self.read_register(Register::GYRO_ZOUT_H)?;

        Ok(I16x3::new(x, y, z))
    }

    pub fn read(&mut self) -> Result<F32x3, E> {
        let raw: F32x3 = self.read_raw()?.into();
        Ok(F32x3::new((raw.x + self.offset.x) / LSB_DEG,
                      (raw.y + self.offset.y) / LSB_DEG,
                      (raw.z + self.offset.z) / LSB_DEG,
                      ))
    }

    fn write_register(&mut self, register: Register, value: u8) -> Result<(), E> {
        self.i2c.write(ADDRESS, &[register.addr(), value])
    }

    fn read_register(&mut self, register: Register) -> Result<i16, E> {
        let mut buffer = [0u8; 2];
        self.i2c.write_read(ADDRESS, &[register.addr()], &mut buffer)?;
        Ok(i16::from_be_bytes(buffer))
    }
}

