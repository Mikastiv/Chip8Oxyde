use rand::Rng;
use sdl2::audio::AudioDevice;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::time::{Duration, Instant};

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
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
    audio_playing: bool,
}

impl Chip8 {
    pub fn new(canvas: Canvas<Window>, audio_device: AudioDevice<SquareWave>) -> Self {
        let mut memory = [0; config::CHIP8_MEMORY_SIZE];
        memory[..config::CHIP8_CHARACTER_SET_SIZE].copy_from_slice(&DEFAULT_CHARACTER_SET[..]);

        Self {
            memory,
            registers: Registers::new(),
            stack: [0; config::CHIP8_STACK_DEPTH],
            keyboard: Keyboard::new(),
            screen: Screen::new(),
            canvas,
            audio_device,
            audio_playing: false,
        }
    }

    pub fn load(&mut self, buf: &[u8]) -> Result<(), Chip8Error> {
        if buf.len() + config::CHIP8_PROGRAM_LOAD_ADDRESS >= config::CHIP8_MEMORY_SIZE {
            return Err(Chip8Error::ProgramTooLarge);
        }

        let start_addr = config::CHIP8_PROGRAM_LOAD_ADDRESS;
        self.memory[start_addr..start_addr + buf.len()].copy_from_slice(buf);

        Ok(())
    }

    pub fn run(&mut self, event_pump: &mut EventPump) {
        let mut dt_duration = Duration::from_secs(0);
        let mut st_duration = Duration::from_secs(0);

        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                config::CHIP8_WIDTH,
                config::CHIP8_HEIGHT,
            )
            .unwrap();

        'running: loop {
            let loop_start = Instant::now();

            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.clear();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => self.keyboard.key_down(key),

                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => self.keyboard.key_up(key),
                    _ => {}
                }
            }

            // Draw frame on a SDL texture
            texture
                .update(
                    None,
                    self.screen.pixel_colors(),
                    config::CHIP8_WIDTH as usize * 3,
                )
                .unwrap();

            // Draw frame texture to window
            self.canvas.copy(&texture, None, None).unwrap();
            self.canvas.present();

            let time_passed = Instant::now() - loop_start;
            dt_duration += time_passed;
            st_duration += time_passed;

            if self.update_sound_timer(st_duration.as_secs_f64()) {
                st_duration = Duration::from_secs(0);
            }

            if self.update_delay_timer(dt_duration.as_secs_f64()) {
                dt_duration = Duration::from_secs(0);
                let opcode = self.get_u16(self.registers.pc);
                self.registers.pc += 2;
                self.exec(opcode, event_pump);
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_character(&mut self, x: usize, y: usize, c: Character) {
        self.screen.draw_sprite(
            x,
            y,
            &self.memory[c as usize..c as usize + config::CHIP8_DEFAULT_SPRITE_HEIGHT as usize],
        );
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

    fn get_u16(&self, addr: u16) -> u16 {
        (self.memory[addr as usize] as u16) << 8 | (self.memory[addr as usize + 1] as u16)
    }

    fn exec(&mut self, opcode: u16, event_pump: &mut EventPump) {
        match opcode {
            0x00E0 => self.cls(),
            0x00EE => self.ret(),
            opcode => self.decode_byte(opcode, event_pump),
        }
    }

    fn decode_byte(&mut self, opcode: u16, event_pump: &mut EventPump) {
        let nnn = opcode & 0x0FFF;
        let n = (opcode & 0x000F) as usize;
        let x = ((opcode >> 8) & 0x000F) as usize;
        let y = ((opcode >> 4) & 0x000F) as usize;
        let kk = (opcode & 0x00FF) as u8;
        match opcode & 0xF000 {
            0x1000 => self.jp(nnn),
            0x2000 => self.call(nnn),
            0x3000 => self.se_vx_byte(x, kk),
            0x4000 => self.sne_vx_byte(x, kk),
            0x5000 if opcode & 0xF == 0x0 => self.se_vx_vy(x, y),
            0x6000 => self.ld_vx_byte(x, kk),
            0x7000 => self.add_vx_byte(x, kk),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.ld_vx_vy(x, y),
                0x0001 => self.or_vx_vy(x, y),
                0x0002 => self.and_vx_vy(x, y),
                0x0003 => self.xor_vx_vy(x, y),
                0x0004 => self.add_vx_vy(x, y),
                0x0005 => self.sub_vx_vy(x, y),
                0x0006 => self.shr_vx(x),
                0x0007 => self.subn_vx_vy(x, y),
                0x000E => self.shl_vx(x),
                _ => panic!("Illegal opcode"),
            },
            0x9000 if opcode & 0xF == 0x0 => self.sne_vx_vy(x, y),
            0xA000 => self.ld_i(nnn),
            0xB000 => self.jp_v0(nnn),
            0xC000 => self.rnd(x, kk),
            0xD000 => self.drw(x, y, n),
            0xE000 => match opcode & 0x00FF {
                0x9E => self.skp(x),
                0xA1 => self.sknp(x),
                _ => panic!("Illegal opcode"),
            },
            0xF000 => match opcode & 0x00FF {
                0x07 => self.ld_vx_dt(x),
                0x0A => self.ld_vx_k(x, event_pump),
                0x15 => self.ld_dt_vx(x),
                0x18 => self.ld_st_vx(x),
                0x1E => self.add_i_vx(x),
                0x29 => self.ld_f_vx(x),
                0x33 => self.ld_b_vx(x),
                0x55 => self.ld_i_vx(x),
                0x65 => self.ld_vx_i(x),
                _ => panic!("Illegal opcode"),
            },
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

    fn wait_for_key(&self, event_pump: &mut EventPump) -> usize {
        for event in event_pump.wait_iter() {
            if let Event::KeyDown {
                keycode: Some(key), ..
            } = event
            {
                if let Some(key) = self.keyboard.map_key(key) {
                    return *key;
                }
            }
        }

        unreachable!();
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
            self.registers.pc = self.registers.pc.wrapping_add(2);
        }
    }

    // 0x4xkk - SNE Vx, byte: Skip next instruction if Vx != kk
    fn sne_vx_byte(&mut self, x: usize, kk: u8) {
        if self.registers.v[x] != kk {
            self.registers.pc = self.registers.pc.wrapping_add(2);
        }
    }

    // 0x5xy0 SE Vx, Vy: Skip next instruction if Vx == Vy
    fn se_vx_vy(&mut self, x: usize, y: usize) {
        if self.registers.v[x] == self.registers.v[y] {
            self.registers.pc = self.registers.pc.wrapping_add(2);
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
            self.registers.pc = self.registers.pc.wrapping_add(2);
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
    fn rnd(&mut self, x: usize, kk: u8) {
        let rng = rand::thread_rng().gen_range(0..=255);
        self.registers.v[x] = rng & kk;
    }

    // 0xDxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn drw(&mut self, x: usize, y: usize, n: usize) {
        let sprite_addr = self.registers.i as usize;
        let collision = self.screen.draw_sprite(
            self.registers.v[x] as usize,
            self.registers.v[y] as usize,
            &self.memory[sprite_addr..sprite_addr + n],
        );

        self.registers.v[0xF] = collision as u8;
    }

    // 0xEx9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed
    fn skp(&mut self, x: usize) {
        if self.keyboard.is_key_down(self.registers.v[x] as usize) {
            self.registers.pc = self.registers.pc.wrapping_add(2);
        }
    }

    // 0xExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed
    fn sknp(&mut self, x: usize) {
        if !self.keyboard.is_key_down(self.registers.v[x] as usize) {
            self.registers.pc = self.registers.pc.wrapping_add(2);
        }
    }

    // 0xFx07 - LD Vx, DT: Set Vx = delay timer value
    fn ld_vx_dt(&mut self, x: usize) {
        self.registers.v[x] = self.registers.dt;
    }

    // 0xFx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx
    fn ld_vx_k(&mut self, x: usize, event_pump: &mut EventPump) {
        let key = self.wait_for_key(event_pump);

        self.registers.v[x] = key as u8;
    }

    // 0xFx15 - LD DT, Vx: Set delay timer = Vx
    fn ld_dt_vx(&mut self, x: usize) {
        self.registers.dt = self.registers.v[x];
    }

    // 0xFx18 - LD ST, Vx: Set sound timer = Vx
    fn ld_st_vx(&mut self, x: usize) {
        self.registers.st = self.registers.v[x];
    }

    // 0xFx1E ADD I, Vx: The values of I and Vx are added, and the results are stored in I
    fn add_i_vx(&mut self, x: usize) {
        self.registers.i = self.registers.i.wrapping_add(self.registers.v[x] as u16);
    }

    // 0xFx29 - LD F, Vx: Set I = sprite address of character in Vx
    fn ld_f_vx(&mut self, x: usize) {
        self.registers.i =
            (self.registers.v[x].wrapping_mul(config::CHIP8_DEFAULT_SPRITE_HEIGHT)) as u16;
    }

    // 0xFx33 LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn ld_b_vx(&mut self, x: usize) {
        let value = self.registers.v[x];
        let units = value % 10;
        let tens = value / 10 % 10;
        let hundreds = value / 100;

        self.memory[self.registers.i as usize] = hundreds;
        self.memory[self.registers.i as usize + 1] = tens;
        self.memory[self.registers.i as usize + 2] = units;
    }

    // 0xFx55 LD [I], Vx: Store registers V0 through Vx in memory starting at location I
    fn ld_i_vx(&mut self, x: usize) {
        let start_loc = self.registers.i as usize;
        self.memory[start_loc..start_loc + x].copy_from_slice(&self.registers.v[..=x]);
    }

    // 0xFx65 LD Vx, [I]: Read registers V0 through Vx from memory starting at location I
    fn ld_vx_i(&mut self, x: usize) {
        let start_loc = self.registers.i as usize;
        self.registers.v[..=x].copy_from_slice(&self.memory[start_loc..start_loc + x]);
    }
}
