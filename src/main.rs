use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use chip8::Chip8;

mod chip8;

fn main() {
    let mut chip8 = Chip8::new();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            chip8::WINDOW_TITLE,
            chip8::CHIP8_WIDTH * chip8::CHIP8_WINDOW_SCALE_FACTOR,
            chip8::CHIP8_HEIGHT * chip8::CHIP8_WINDOW_SCALE_FACTOR,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // let texture_creator = canvas.texture_creator();
    // let mut screen = texture_creator
    //     .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
    //     .unwrap();

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Draw frame on a SDL texture
        // screen
        //     .update(None, cpu.screen(), SCREEN_WIDTH as usize * 3)
        //     .unwrap();

        // Draw frame texture to window
        // canvas.copy(&screen, None, screen_rect).unwrap();
        let screen_rect = Rect::new(0, 0, 40, 40);
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rect(screen_rect).unwrap();

        canvas.present();
    }

    println!("Hello, world!");
}
