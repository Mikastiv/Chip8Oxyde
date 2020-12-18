use crate::config;
use config::CHIP8_STACK_DEPTH;
use keyboard::Keyboard;
use registers::Registers;

mod keyboard;
mod registers;

pub struct Chip8 {
    pub memory: [u8; config::CHIP8_MEMORY_SIZE],
    pub registers: Registers,
    stack: [u16; config::CHIP8_STACK_DEPTH],
    pub keyboard: Keyboard,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            memory: [0; config::CHIP8_MEMORY_SIZE],
            registers: Registers::new(),
            stack: [0; CHIP8_STACK_DEPTH],
            keyboard: Keyboard::new(),
        }
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.registers.sp as usize] = val;
        self.registers.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.registers.sp -= 1;
        self.stack[self.registers.sp as usize]
    }
}
