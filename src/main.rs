use clap::{App, Arg};
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use std::fs::File;
use std::io::{Error, Read};
use std::time::{Duration, Instant};

use chip8::character::Character;
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

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            config::CHIP8_WIDTH,
            config::CHIP8_HEIGHT,
        )
        .unwrap();

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

    let mut chip8 = Chip8::new(audio_device);
    chip8.load(&program_buffer).unwrap();

    chip8.draw_character(0, 0, Character::Num0);
    chip8.draw_character(8, 0, Character::Num1);
    chip8.draw_character(16, 0, Character::Num2);
    chip8.draw_character(24, 0, Character::Num3);
    chip8.draw_character(0, 8, Character::Num4);
    chip8.draw_character(8, 8, Character::Num5);
    chip8.draw_character(16, 8, Character::Num6);
    chip8.draw_character(24, 8, Character::Num7);
    chip8.draw_character(0, 16, Character::Num8);
    chip8.draw_character(8, 16, Character::Num9);
    chip8.draw_character(16, 16, Character::A);
    chip8.draw_character(24, 16, Character::B);
    chip8.draw_character(0, 24, Character::C);
    chip8.draw_character(8, 24, Character::D);
    chip8.draw_character(16, 24, Character::E);
    chip8.draw_character(24, 24, Character::F);

    let mut dt_duration = Duration::from_secs(0);
    let mut st_duration = Duration::from_secs(0);

    'running: loop {
        let loop_start = Instant::now();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => chip8.keyboard_mut().key_down(key),

                Event::KeyUp {
                    keycode: Some(key), ..
                } => chip8.keyboard_mut().key_up(key),
                _ => {}
            }
        }

        // Draw frame on a SDL texture
        texture
            .update(
                None,
                chip8.screen().pixel_colors(),
                config::CHIP8_WIDTH as usize * 3,
            )
            .unwrap();

        // Draw frame texture to window
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        let time_passed = Instant::now() - loop_start;
        dt_duration += time_passed;
        st_duration += time_passed;

        if chip8.update_sound_timer(st_duration.as_secs_f64()) {
            st_duration = Duration::from_secs(0);
        }

        if chip8.update_delay_timer(dt_duration.as_secs_f64()) {
            dt_duration = Duration::from_secs(0);
            let opcode = chip8.get_u16(chip8.registers.pc);
            chip8.registers.pc += 2;
            chip8.exec(opcode);
        }
    }
}
