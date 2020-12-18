pub const WINDOW_TITLE: &str = "Chip8 Window";
pub const CHIP8_MEMORY_SIZE: usize = 4096;
pub const CHIP8_WIDTH: u32 = 64;
pub const CHIP8_HEIGHT: u32 = 32;
pub const CHIP8_WINDOW_SCALE_FACTOR: u32 = 10;

pub struct Chip8 {
    pub memory: [u8; CHIP8_MEMORY_SIZE],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            memory: [0; CHIP8_MEMORY_SIZE],
        }
    }
}
