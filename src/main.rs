use std::time::Duration;
use std::u32;

use chip8::{Chip8, StepMode};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

struct Opts {
    mode: StepMode,
    file: String,
    tickrate: u32,
    fg: u32,
    bg: u32,
}

// TODO this sucks
fn parse_cl() -> Result<Opts, String> {
    let args: Vec<String> = std::env::args().collect();
    let mode = match args.iter().position(|e| e == "-m" || e == "--mode") {
        Some(idx) => match args.get(idx + 1) {
            Some(val) => match val.as_str() {
                "d" | "debug" => StepMode::Debug,
                "c" | "cycle" | _ => StepMode::Cycle,
            },
            None => StepMode::Cycle,
        },
        None => StepMode::Cycle,
    };

    let file = match args.iter().position(|e| e == "-f" || e == "--file") {
        Some(idx) => match args.get(idx + 1) {
            Some(val) => val,
            None => return Err("Found --file option, but no file name".into()),
        },
        None => return Err("Missing required option filename".into()),
    };

    let tickrate = match args.iter().position(|e| e == "-t" || e == "--time") {
        Some(idx) => match args.get(idx + 1) {
            Some(val) => val.parse::<u32>().map_err(|e| e.to_string())?,
            None => return Err("Found --time option, but no time value".into()),
        },
        None => 20,
    };

    let fg = match args.iter().position(|e| e == "--fg") {
        Some(idx) => match args.get(idx + 1) {
            Some(val) => u32::from_str_radix(val, 16).unwrap(),
            None => return Err("Found --fg option, but no color val".into()),
        },
        None => 0xffffff,
    };

    let bg = match args.iter().position(|e| e == "--bg") {
        Some(idx) => match args.get(idx + 1) {
            Some(val) => u32::from_str_radix(val, 16).unwrap(),
            None => return Err("Found --bg option, but no color value".into()),
        },
        None => 0x0,
    };

    let file = file.to_owned();
    Ok(Opts {
        mode,
        file,
        tickrate,
        fg,
        bg,
    })
}

fn main() -> Result<(), String> {
    // init gfx and key read contexts

    let opts = parse_cl()?;

    let win_width = 1024;
    let win_height = 512;
    let sdl_context = sdl2::init()?;
    let video_context = sdl_context.video()?;
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
    let bg = create_colored_rect(&creator, opts.bg, tex_h, tex_w)?;
    let fg = create_colored_rect(&creator, opts.fg, tex_h, tex_w)?;
    let mut render_rect = Rect::new(1, 1, tex_w as u32, tex_h as u32);

    let mut comp = Chip8::new().with_mode(opts.mode);
    let prog = std::fs::read(opts.file).map_err(|e| e.to_string())?;
    comp.load(&prog)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut user_break = false;
    'render: while comp.running {
        let mut i = 0;
        while i < opts.tickrate {
            if comp.draw {
                i = opts.tickrate;
            }
            match comp.step() {
                Ok(_) => {}
                Err(e) => {
                    return Err(format!("emu step fail: {} on cc {}", e, comp.cycles));
                }
            }

            i += 1;
        }

        comp.dec_timers();

        if comp.draw {
            for i in 0..comp.gfx.len() {
                let y = (i / 64) * tex_h as usize;
                let x = (i % 64) * tex_w as usize;

                render_rect.set_x(x as i32);
                render_rect.set_y(y as i32);

                if comp.gfx[i] == 1 {
                    canvas.copy(&fg, None, Some(render_rect))?;
                } else {
                    canvas.copy(&bg, None, Some(render_rect))?
                }
            }
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
                    user_break = true;
                    break 'render;
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    handle_key(&mut comp, key, false);
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    handle_key(&mut comp, key, true);
                }
                _ => {}
            }
        }

        // naively target 60 fps
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    println!(
        "{} {} cycles",
        if user_break {
            "Stopped after"
        } else {
            "Completed in"
        },
        comp.cycles
    );

    Ok(())
}

fn _debug_render(gfx: &[u8]) {
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

// TODO still don't like this but better
fn handle_key(comp: &mut Chip8, key: Keycode, keyup: bool) {
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

    if key > 15 {
        return;
    }

    if keyup {
        comp.key_up(key)
    } else {
        comp.key_down(key)
    }
}

fn create_colored_rect(
    creator: &TextureCreator<WindowContext>,
    c: u32,
    tex_h: u32,
    tex_w: u32,
) -> Result<Texture, String> {
    let mut tex = creator
        .create_texture_streaming(PixelFormatEnum::RGB24, tex_w, tex_h)
        .map_err(|e| e.to_string())?;

    let b = (c & 0xff) as u8;
    let g = (c >> 8 & 0xff) as u8;
    let r = (c >> 16 & 0xff) as u8;

    tex.with_lock(None, |buf: &mut [u8], pitch: usize| {
        for y in 0..tex_h {
            for x in 0..tex_w {
                let offset = y as usize * pitch + x as usize * 3;
                buf[offset] = r;
                buf[offset + 1] = g;
                buf[offset + 2] = b;
            }
        }
    })?;

    Ok(tex)
}
