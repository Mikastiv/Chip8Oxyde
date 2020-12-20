use clap::{App, Arg};
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::fs::File;
use std::io::{Error, Read};
use std::time::{Duration, Instant};

use chip8::Chip8;

mod chip8;
mod config;

fn read_file(file_path: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(file_path)?;
    let size = file.metadata()?.len();
    let mut buffer = Vec::with_capacity(size as usize);

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn main() {
    let matches = App::new("Chip8Oxyde")
        .author("Mikastiv <m.leblanc_3@hotmail.com>")
        .about("Chip8 emulator written in Rust")
        .version("0.1.0")
        .arg(
            Arg::with_name("Program file")
                .help("Program to load")
                .index(1)
                .required(true),
        )
        .get_matches();

    let program_file = matches.value_of("Program file").unwrap();
    let program_buffer = read_file(program_file).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let window = video_subsystem
        .window(
            config::WINDOW_TITLE,
            config::CHIP8_WIDTH * config::CHIP8_WINDOW_SCALE_FACTOR,
            config::CHIP8_HEIGHT * config::CHIP8_WINDOW_SCALE_FACTOR,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let audio_device = audio_subsystem
        .open_playback(None, &audio_spec, |spec| chip8::audio::SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.05,
        })
        .unwrap();

    let mut chip8 = Chip8::new(canvas, audio_device);
    chip8.load(&program_buffer).unwrap();
    chip8.run(&mut event_pump);
}
