use crate::config;
use character::{Character, DEFAULT_CHARACTER_SET};
use keyboard::Keyboard;
use pc_beeper::Speaker;
use registers::Registers;
use screen::Screen;

pub mod character;
mod keyboard;
mod registers;
mod screen;

#[derive(Debug)]
pub struct Chip8 {
    pub memory: [u8; config::CHIP8_MEMORY_SIZE],
    pub registers: Registers,
    stack: [u16; config::CHIP8_STACK_DEPTH],
    keyboard: Keyboard,
    screen: Screen,
    speaker: Speaker,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; config::CHIP8_MEMORY_SIZE];
        memory[..config::CHIP8_CHARACTER_SET_SIZE].copy_from_slice(&DEFAULT_CHARACTER_SET[..]);

        Self {
            memory,
            registers: Registers::new(),
            stack: [0; config::CHIP8_STACK_DEPTH],
            keyboard: Keyboard::new(),
            screen: Screen::new(),
            speaker: Speaker::new(),
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

    pub fn draw_character(&mut self, x: usize, y: usize, c: Character) {
        self.screen
            .draw_sprite(x, y, &self.memory[c as usize..c as usize + 5]);
    }

    pub fn update_delay_timer(&mut self, delta: f64) -> bool {
        if delta >= config::CHIP8_DELAY_TIMER_FREQ && self.registers.dt > 0 {
            self.registers.dt -= 1;
            return true;
        }

        false
    }

    pub fn update_sound_timer(&mut self, delta: f64) -> bool {
        if delta >= config::CHIP8_SOUND_TIMER_FREQ && self.registers.st > 0 {
            let duration =
                (config::CHIP8_SOUND_TIMER_FREQ / 1000.0).round() as u32 * self.registers.st as u32;
            self.speaker.beep(12000, duration);
            self.registers.st = 0;
            return true;
        }

        false
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }
}
