use crate::config;

#[derive(Debug)]
pub struct Screen {
    pixel_states: [bool; config::CHIP8_WIDTH as usize * config::CHIP8_HEIGHT as usize],
    pixels_for_draw: [u8; config::CHIP8_WIDTH as usize * config::CHIP8_HEIGHT as usize * 3],
}

impl Screen {
    pub fn new() -> Self {
        Self {
            pixel_states: [false; config::CHIP8_WIDTH as usize * config::CHIP8_HEIGHT as usize],
            pixels_for_draw: [0; config::CHIP8_WIDTH as usize * config::CHIP8_HEIGHT as usize * 3],
        }
    }

    pub fn pixel_colors(&self) -> &[u8] {
        &self.pixels_for_draw[..]
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut pixel_erased = false;

        for (ly, c) in sprite.iter().enumerate() {
            for lx in 0..8 {
                if *c & (0x80 >> lx) == 0 {
                    continue;
                }

                let index_x = (lx + x) % config::CHIP8_WIDTH as usize;
                let index_y = (ly + y) % config::CHIP8_HEIGHT as usize;

                if self.pixel(index_x, index_y) {
                    pixel_erased = true;
                }

                self.set_pixel(index_x, index_y, true);
            }
        }

        pixel_erased
    }

    pub fn clear(&mut self) {
        for val in self.pixel_states.iter_mut() {
            *val = false;
        }

        for val in self.pixels_for_draw.iter_mut() {
            *val = 0x00;
        }
    }

    fn pixel(&self, x: usize, y: usize) -> bool {
        self.pixel_states[Self::convert_2d_to_1d(x, y)]
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: bool) {
        for i in 0..3 {
            let index = Screen::convert_2d_to_1d_for_draw(x, y);
            if val {
                self.pixels_for_draw[index + i] ^= 0xFF;
            } else {
                self.pixels_for_draw[index + i] ^= 0x00;
            }
        }

        self.pixel_states[Self::convert_2d_to_1d(x, y)] ^= val;
    }

    fn convert_2d_to_1d(x: usize, y: usize) -> usize {
        y * config::CHIP8_WIDTH as usize + x
    }

    fn convert_2d_to_1d_for_draw(x: usize, y: usize) -> usize {
        y * config::CHIP8_WIDTH as usize * 3 + x * 3
    }
}
