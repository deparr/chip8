use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use chip8::Chip8;


fn main() {
    // init gfx and key read contexts
    let sdl_context = sdl2::init().unwrap();
    let video_context = sdl_context.video().unwrap();
    let window = video_context.window("chip8", 400, 200)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let surface = Surface::new(400, 200, PixelFormatEnum::RGB24).unwrap();
    let creator = canvas.texture_creator();
    let tex = Texture::from_surface(&surface, &creator);

    let mut comp = Chip8::new().with_mode(chip8::StepMode::Debug);
    let prog = std::fs::read("./roms/pong2.c8").unwrap();
    comp.load(&prog).unwrap();
    let mut cc = 0;
    'render: loop {
        match comp.step() {
            Ok(_) => {}
            Err(e) => {
                panic!("emu step fail: {} on cc {}", e, cc + 1);
            }
        }
        cc += 1;

        if comp.draw {
            // if comp.step_mode == chip8::StepMode::Debug {
            //     debug_render(&comp.gfx);
            // }
            comp.draw = false;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'render;
                }
                Event::KeyDown { keycode: Some(key), repeat: false, .. } => {
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
                },
                Event::KeyUp { keycode: Some(key), .. } => {
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
                },
                _  => {},

            }
        }
        canvas.present();
    }
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
