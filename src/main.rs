use chip8::Chip8;

fn main() {
    // init gfx and key read contexts
    let mut comp = Chip8::new();
    // load prog
    loop {
        match comp.step() {
            Ok(_) => {},
            Err(e) => {
                panic!("emu step fail: {}", e)
            }
        }

        if comp.draw {
            // redraw gfx
        }

        // set keys
    }

}
