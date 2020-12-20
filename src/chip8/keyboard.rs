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
            key_map: config::create_key_map(),
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

    pub fn is_key_down(&self, key: usize) -> bool {
        self.key_states[key]
    }

    pub fn map_key(&self, key: Keycode) -> Option<&usize> {
        self.key_map.get(&key)
    }
}


