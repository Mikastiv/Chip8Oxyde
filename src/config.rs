use sdl2::keyboard::Keycode;
use std::collections::HashMap;

pub const WINDOW_TITLE: &str = "Chip8 Window";
pub const CHIP8_MEMORY_SIZE: usize = 0x1000;
pub const CHIP8_PROGRAM_LOAD_ADDRESS: usize = 0x200;
pub const CHIP8_WIDTH: u32 = 64;
pub const CHIP8_HEIGHT: u32 = 32;
pub const CHIP8_WINDOW_SCALE_FACTOR: u32 = 20;
pub const CHIP8_DATA_REGISTER_COUNT: usize = 16;
pub const CHIP8_STACK_DEPTH: usize = 16;
pub const CHIP8_KEY_COUNT: usize = 16;
pub const CHIP8_CHARACTER_SET_SIZE: usize = 80;
pub const CHIP8_DELAY_TIMER_FREQ: f64 = 1.0 / 60.0;
pub const CHIP8_SOUND_TIMER_FREQ: f64 = 1.0 / 60.0;
pub const CHIP8_EXEC_FREQ: f64 = 1.0 / 800.0;
pub const CHIP8_DEFAULT_SPRITE_HEIGHT: u8 = 5;
pub const CHIP8_SOUND_NOTE_FREQ: f32 = 440.0;

pub fn create_key_map() -> HashMap<Keycode, usize> {
    let mut map = HashMap::with_capacity(CHIP8_KEY_COUNT);

    map.insert(Keycode::Num0, 0);
    map.insert(Keycode::Num1, 1);
    map.insert(Keycode::Num2, 2);
    map.insert(Keycode::Num3, 3);
    map.insert(Keycode::Num4, 4);
    map.insert(Keycode::Num5, 5);
    map.insert(Keycode::Num6, 6);
    map.insert(Keycode::Num7, 7);
    map.insert(Keycode::Num8, 8);
    map.insert(Keycode::Num9, 9);
    map.insert(Keycode::A, 10);
    map.insert(Keycode::B, 11);
    map.insert(Keycode::C, 12);
    map.insert(Keycode::D, 13);
    map.insert(Keycode::E, 14);
    map.insert(Keycode::F, 15);

    map
}