use std::error::Error;

use crate::config;
use audio::SquareWave;
use character::{Character, DEFAULT_CHARACTER_SET};
use error::Chip8Error;
use keyboard::Keyboard;
use registers::Registers;
use screen::Screen;
use sdl2::audio::AudioDevice;

pub mod audio;
pub mod character;
pub mod error;
mod keyboard;
mod registers;
mod screen;

pub struct Chip8 {
    pub memory: [u8; config::CHIP8_MEMORY_SIZE],
    pub registers: Registers,
    stack: [u16; config::CHIP8_STACK_DEPTH],
    keyboard: Keyboard,
    screen: Screen,
    audio_device: AudioDevice<SquareWave>,
    audio_playing: bool,
}

impl Chip8 {
    pub fn new(audio_device: AudioDevice<SquareWave>) -> Self {
        let mut memory = [0; config::CHIP8_MEMORY_SIZE];
        memory[..config::CHIP8_CHARACTER_SET_SIZE].copy_from_slice(&DEFAULT_CHARACTER_SET[..]);

        Self {
            memory,
            registers: Registers::new(),
            stack: [0; config::CHIP8_STACK_DEPTH],
            keyboard: Keyboard::new(),
            screen: Screen::new(),
            audio_device,
            audio_playing: false,
        }
    }

    pub fn exec(&mut self, opcode: u16) {}

    pub fn load(&mut self, buf: &[u8]) -> Result<(), Chip8Error> {
        if buf.len() + config::CHIP8_PROGRAM_LOAD_ADDRESS >= config::CHIP8_MEMORY_SIZE {
            return Err(Chip8Error::ProgramTooLarge);
        }

        self.memory
            [config::CHIP8_PROGRAM_LOAD_ADDRESS..config::CHIP8_PROGRAM_LOAD_ADDRESS + buf.len()]
            .copy_from_slice(buf);

        Ok(())
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
            if !self.audio_playing {
                self.audio_playing = true;
                self.audio_device.resume();
            }

            self.registers.st -= 1;
            return true;
        }

        if self.audio_playing && self.registers.st == 0 {
            self.audio_playing = false;
            self.audio_device.pause();
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
