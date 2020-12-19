use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

use chip8::character::Character;
use chip8::Chip8;

mod chip8;
mod config;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.registers.dt = 255;

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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
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

    let mut dt_duration = std::time::Duration::from_secs(0);
    let mut st_duration = std::time::Duration::from_secs(0);

    'running: loop {
        let loop_start = std::time::Instant::now();

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

        let time_passed = std::time::Instant::now() - loop_start;
        dt_duration += time_passed;
        st_duration += time_passed;

        if chip8.update_delay_timer(dt_duration.as_secs_f64()) {
            dt_duration = std::time::Duration::from_secs(0);
        }

        if chip8.update_sound_timer(st_duration.as_secs_f64()) {
            st_duration = std::time::Duration::from_secs(0);
        }
    }
}
