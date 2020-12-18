use crate::config::CHIP8_DATA_REGISTER_COUNT;

pub struct Registers {
    pub v: [u8; CHIP8_DATA_REGISTER_COUNT],
    pub i: u16,
    pub dt: u8,
    pub st: u8,
    pub pc: u16,
    pub sp: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            v: [0; CHIP8_DATA_REGISTER_COUNT],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0,
            sp: 0,
        }
    }
}
