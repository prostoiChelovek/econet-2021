#![allow(non_camel_case_types, clippy::unreadable_literal)]

#[repr(u8)]
pub enum Register {
    PWR_MGM = 0x3E,
    SMPLRT_DIV = 0x15,
    DLPF_FS = 0x16,
    INT_CFG = 0x17,

    TEMP_OUT_H = 0x1B,
    TEMP_OUT_L = 0x1C,

    GYRO_XOUT_H = 0x1D,
    GYRO_XOUT_L = 0x1E,

    GYRO_YOUT_H = 0x1F,
    GYRO_YOUT_L = 0x20,

    GYRO_ZOUT_H = 0x21,
    GYRO_ZOUT_L = 0x22,
}

impl Register {
    /// Get register address
    pub fn addr(self) -> u8 {
        self as u8
    }
}

