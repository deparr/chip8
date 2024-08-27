use sdl2::pixels::Color;
use chip8::Chip8;


fn main() {
    // init gfx and key read contexts
    let sdl_context = sdl2::init().unwrap();
    let video_context = sdl_context.video().unwrap();
    let window = video_context.window("chip8", 200, 100)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut comp = Chip8::new().with_mode(chip8::StepMode::Debug);
    let prog = std::fs::read("./roms/pong2.c8").unwrap();
    comp.load(&prog).unwrap();
    let mut cc = 0;
    loop {
        match comp.step() {
            Ok(_) => {}
            Err(e) => {
                panic!("emu step fail: {} on cc {}", e, cc+1);
            }
        }
        cc += 1;

        if comp.draw {
            if comp.step_mode == chip8::StepMode::Debug {
                debug_render(&comp.gfx);
            }
            comp.draw = false;
        }

        // set keys
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
