use sdl2::audio::AudioDevice;
use rand::Rng;

use crate::config;
use audio::SquareWave;
use character::{Character, DEFAULT_CHARACTER_SET};
use error::Chip8Error;
use keyboard::Keyboard;
use registers::Registers;
use screen::Screen;

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

    pub fn load(&mut self, buf: &[u8]) -> Result<(), Chip8Error> {
        if buf.len() + config::CHIP8_PROGRAM_LOAD_ADDRESS >= config::CHIP8_MEMORY_SIZE {
            return Err(Chip8Error::ProgramTooLarge);
        }

        self.memory
            [config::CHIP8_PROGRAM_LOAD_ADDRESS..config::CHIP8_PROGRAM_LOAD_ADDRESS + buf.len()]
            .copy_from_slice(buf);

        Ok(())
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

    pub fn get_u16(&self, addr: u16) -> u16 {
        (self.memory[addr as usize] as u16) << 8 | (self.memory[addr as usize + 1] as u16)
    }

    pub fn exec(&mut self, opcode: u16) {
        match opcode {
            0x00E0 => self.cls(),
            0x00EE => self.ret(),
            opcode => self.decode_byte(opcode),
        }
    }

    fn decode_byte(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        let n = opcode & 0x000F;
        let x = (opcode >> 8) & 0x000F;
        let y = (opcode >> 4) & 0x000F;
        let kk = (opcode & 0x00FF) as u8;
        match opcode & 0xF000 {
            0x1000 => self.jp(nnn),
            0x2000 => self.call(nnn),
            0x3000 => self.se_vx_byte(x as usize, kk),
            0x4000 => self.sne_vx_byte(x as usize, kk),
            0x5000 if opcode & 0x000F == 0 => self.se_vx_vy(x as usize, y as usize),
            0x6000 => self.ld_vx_byte(x as usize, kk),
            0x7000 => self.add_vx_byte(x as usize, kk),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.ld_vx_vy(x as usize, y as usize),
                0x0001 => self.or_vx_vy(x as usize, y as usize),
                0x0002 => self.and_vx_vy(x as usize, y as usize),
                0x0003 => self.xor_vx_vy(x as usize, y as usize),
                0x0004 => self.add_vx_vy(x as usize, y as usize),
                0x0005 => self.sub_vx_vy(x as usize, y as usize),
                0x0006 => self.shr_vx(x as usize),
                0x0007 => self.subn_vx_vy(x as usize, y as usize),
                0x000E => self.shl_vx(x as usize),
                _ => panic!("Illegal opcode"),
            },
            0x9000 if opcode & 0x000F == 0 => self.sne_vx_vy(x as usize, y as usize),
            0xA000 => self.ld_i(nnn),
            0xB000 => self.jp_v0(nnn),
            0xC000 => self.rnd_vx(x as usize, kk),
            0xD000 => {}
            0xE000 => {}
            0xF000 => {}
            _ => panic!("Illegal opcode"),
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.registers.sp as usize] = val;
        self.registers.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.registers.sp -= 1;
        self.stack[self.registers.sp as usize]
    }

    // CLS: Clear the screen
    fn cls(&mut self) {
        self.screen.clear();
    }

    // RET: Return from subroutine
    fn ret(&mut self) {
        self.registers.pc = self.pop();
    }

    // 0x1nnn - JP: Jump to location addr
    fn jp(&mut self, nnn: u16) {
        self.registers.pc = nnn;
    }

    // 0x2nnn CALL: Call subroutine at addr
    fn call(&mut self, nnn: u16) {
        self.push(self.registers.pc);
        self.registers.pc = nnn;
    }

    // 0x3xkk - SE Vx, byte: Skip next instruction if Vx == kk
    fn se_vx_byte(&mut self, x: usize, kk: u8) {
        if self.registers.v[x] == kk {
            self.registers.pc += 2;
        }
    }

    // 0x4xkk - SNE Vx, byte: Skip next instruction if Vx != kk
    fn sne_vx_byte(&mut self, x: usize, kk: u8) {
        if self.registers.v[x] != kk {
            self.registers.pc += 2;
        }
    }

    // 0x5xy0 SE Vx, Vy: Skip next instruction if Vx == Vy
    fn se_vx_vy(&mut self, x: usize, y: usize) {
        if self.registers.v[x] == self.registers.v[y] {
            self.registers.pc += 2;
        }
    }

    // 0x6xkk - LD Vx, byte: Load kk into Vx
    fn ld_vx_byte(&mut self, x: usize, kk: u8) {
        self.registers.v[x] = kk;
    }

    // 0x7xkk - ADD Vx, byte: Add kk to Vx
    fn add_vx_byte(&mut self, x: usize, kk: u8) {
        self.registers.v[x] = self.registers.v[x].wrapping_add(kk);
    }

    // 0x8xy0 - LD Vx, Vy: Load Vy into Vx
    fn ld_vx_vy(&mut self, x: usize, y: usize) {
        self.registers.v[x] = self.registers.v[y];
    }

    // 0x8xy1 - OR Vx, Vy: Bitwise OR on Vx and Vy, store result in Vx
    fn or_vx_vy(&mut self, x: usize, y: usize) {
        self.registers.v[x] |= self.registers.v[y];
    }

    // 0x8xy2 - AND Vx, Vy: Bitwise AND on Vx and Vy, store result in Vx
    fn and_vx_vy(&mut self, x: usize, y: usize) {
        self.registers.v[x] &= self.registers.v[y];
    }

    // 0x8xy3 - XOR Vx, Vy: Bitwise XOR on Vx and Vy, store result in Vx
    fn xor_vx_vy(&mut self, x: usize, y: usize) {
        self.registers.v[x] ^= self.registers.v[y];
    }

    // 0x8xy4 - ADD Vx, Vy: Add Vx and Vy, store result in Vx, VF is set to carry bit
    fn add_vx_vy(&mut self, x: usize, y: usize) {
        let result = self.registers.v[x] as u16 + self.registers.v[y] as u16;
        self.registers.v[x] = result as u8;
        self.registers.v[0xF] = (result > 0xFF) as u8;
    }

    // 0x8xy5 - SUB Vx, Vy: Sub Vy from Vx, store result in Vx, VF is set to Vx > Vy
    fn sub_vx_vy(&mut self, x: usize, y: usize) {
        self.registers.v[0xF] = (self.registers.v[x] > self.registers.v[y]) as u8;
        self.registers.v[x] = self.registers.v[x].wrapping_sub(self.registers.v[y]);
    }

    // 0x8xy6 - SHR Vx: Bitwise shift right by 1, VF is set to lowest bit
    fn shr_vx(&mut self, x: usize) {
        self.registers.v[0xF] = self.registers.v[x] & 0x01;
        self.registers.v[x] >>= 1;
    }

    // 0x8xy7 - SUBN Vx, Vy: Sub Vx from Vy, store result in Vx, VF is set to Vy > Vx
    fn subn_vx_vy(&mut self, x: usize, y: usize) {
        self.registers.v[0xF] = (self.registers.v[y] > self.registers.v[x]) as u8;
        self.registers.v[x] = self.registers.v[y].wrapping_sub(self.registers.v[x]);
    }

    // 0x8xyE - SHL Vx: Bitwise shift left by 1, VF is set to highest bit
    fn shl_vx(&mut self, x: usize) {
        self.registers.v[0xF] = self.registers.v[x] & 0x80;
        self.registers.v[x] <<= 1;
    }

    // 0x9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy
    fn sne_vx_vy(&mut self, x: usize, y: usize) {
        if self.registers.v[x] != self.registers.v[y] {
            self.registers.pc += 2;
        }
    }

    // 0xAnnn - LD I, addr: Load nnn into register I
    fn ld_i(&mut self, nnn: u16) {
        self.registers.i = nnn;
    }

    // 0xBnnn - JP V0, addr: Jump to location addr + V0
    fn jp_v0(&mut self, nnn: u16) {
        self.registers.pc = nnn.wrapping_add(self.registers.v[0x0] as u16);
    }

    // 0xCxkk - RND Vx, byte: Generate random number between 0 and 255, then bitwise AND with kk
    fn rnd_vx(&mut self, x:usize, kk: u8) {
        let rng = rand::thread_rng().gen_range(0..=255);
        self.registers.v[x] = rng & kk;
    }
}
