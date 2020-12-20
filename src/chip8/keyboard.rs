use sdl2::keyboard::Keycode;
use std::collections::HashMap;

use crate::config;

#[derive(Debug)]
pub struct Keyboard {
    key_states: [bool; config::CHIP8_KEY_COUNT],
    key_map: HashMap<Keycode, usize>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            key_states: [false; config::CHIP8_KEY_COUNT],
            key_map: create_key_map(),
        }
    }

    pub fn key_down(&mut self, key: Keycode) {
        if let Some(key) = self.map_key(key) {
            self.key_states[*key] = true;
        }
    }

    pub fn key_up(&mut self, key: Keycode) {
        if let Some(key) = self.map_key(key) {
            self.key_states[*key] = false;
        }
    }

    pub fn is_key_down(&self, key: Keycode) -> Option<bool> {
        if let Some(key) = self.map_key(key) {
            return Some(self.key_states[*key]);
        }

        None
    }

    fn map_key(&self, key: Keycode) -> Option<&usize> {
        self.key_map.get(&key)
    }
}

fn create_key_map() -> HashMap<Keycode, usize> {
    let mut map = HashMap::with_capacity(config::CHIP8_KEY_COUNT);

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
