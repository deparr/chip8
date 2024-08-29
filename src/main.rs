use std::time::Duration;

use chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;

fn main() -> Result<(), String> {
    // init gfx and key read contexts
    let win_width = 1024;
    let win_height = 512;
    let sdl_context = sdl2::init().unwrap();
    let video_context = sdl_context.video().unwrap();
    let window = video_context
        .window("chip8", win_width, win_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let tex_w = win_width / 64;
    let tex_h = win_height / 32;
    let mut black = creator
        .create_texture_streaming(PixelFormatEnum::RGB24, tex_w, tex_h)
        .map_err(|e| e.to_string())?;
    black.with_lock(None, |buf: &mut [u8], pitch: usize| {
        for y in 0..tex_h {
            for x in 0..tex_w {
                let offset = y as usize * pitch + x as usize * 3;
                buf[offset] = 0;
                buf[offset + 1] = 0;
                buf[offset + 2] = 0;
            }
        }
    })?;
    let mut white = creator
        .create_texture_streaming(PixelFormatEnum::RGB24, tex_w as u32, tex_h as u32)
        .map_err(|e| e.to_string())?;
    white.with_lock(None, |buf: &mut [u8], pitch: usize| {
        for y in 0..tex_h {
            for x in 0..tex_w {
                let offset = y as usize * pitch + x as usize * 3;
                buf[offset] = 255;
                buf[offset + 1] = 255;
                buf[offset + 2] = 255;
            }
        }
    })?;

    let mut render_rect = Rect::new(1,1, tex_w as u32, tex_h as u32);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut comp = Chip8::new().with_mode(chip8::StepMode::Debug);
    let prog = std::fs::read("./roms/invaders.c8").unwrap();
    comp.load(&prog).unwrap();
    let mut cc = 0;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'render: loop {
        match comp.step() {
            Ok(_) => {}
            Err(e) => {
                panic!("emu step fail: {} on cc {}", e, cc + 1);
            }
        }
        cc += 1;

        if comp.draw {
            canvas.clear();
            for i in 0..comp.gfx.len() {
                let y = (i / 64) * tex_h as usize;
                let x = (i % 64) * tex_w as usize;

                render_rect.set_x(x as i32);
                render_rect.set_y(y as i32);

                if comp.gfx[i] == 1 {
                    canvas.copy(&white, None, Some(render_rect)).expect("render err: ");
                } else {
                    canvas.copy(&black, None, Some(render_rect)).expect("render err: ");
                }
            }
            debug_render(&comp.gfx);
            comp.draw = false;
            canvas.present();
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'render;
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    let key = match key {
                        Keycode::Num1 => 0x1,
                        Keycode::Num2 => 0x2,
                        Keycode::Num3 => 0x3,
                        Keycode::Num4 => 0xc,

                        Keycode::Q => 0x4,
                        Keycode::W => 0x5,
                        Keycode::E => 0x6,
                        Keycode::R => 0xd,

                        Keycode::A => 0x7,
                        Keycode::S => 0x8,
                        Keycode::D => 0x9,
                        Keycode::F => 0xe,

                        Keycode::Z => 0xa,
                        Keycode::X => 0x0,
                        Keycode::C => 0xb,
                        Keycode::V => 0xf,
                        _ => 16,
                    };

                    // Todo dont like this
                    if key <= 15 {
                        comp.key_down(key);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let key = match key {
                        Keycode::Num1 => 0x1,
                        Keycode::Num2 => 0x2,
                        Keycode::Num3 => 0x3,
                        Keycode::Num4 => 0xc,

                        Keycode::Q => 0x4,
                        Keycode::W => 0x5,
                        Keycode::E => 0x6,
                        Keycode::R => 0xd,

                        Keycode::A => 0x7,
                        Keycode::S => 0x8,
                        Keycode::D => 0x9,
                        Keycode::F => 0xe,

                        Keycode::Z => 0xa,
                        Keycode::X => 0x0,
                        Keycode::C => 0xb,
                        Keycode::V => 0xf,
                        _ => 16,
                    };

                    if key <= 15 {
                        comp.key_up(key);
                    }
                }
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    };
    
    Ok(())
}

fn debug_render(gfx: &[u8]) {
    for y in 0..32 {
        for x in 0..64 {
            if gfx[x + y * 64] > 0 {
                print!("⬜");
            } else {
                print!("⬛");
            }
        }
        println!();
    }
    println!();
}
