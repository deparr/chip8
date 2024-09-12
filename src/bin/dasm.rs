use chip8::{Chip8, OpCode};


fn main() -> Result<(), String> {
    let prog = std::env::args().nth(1).expect("expected program path as first arg");
    let prog = std::fs::read(prog).map_err(|e| e.to_string())?;

    let mut i = 0;
    let mut ops: Vec<(OpCode, u16)> = vec![];
    while i < prog.len() {
        let next = ((prog[i] as u16) << 8) | (prog[i+1] as u16);
        ops.push((Chip8::decode(next), next));
        i += 2;
    }


    for i in 0..ops.len() {
        let addr = i * 2;
        println!("{:06x}:\t{:04x}\t|\t{}", addr, ops[i].1, ops[i].0);
    }

    Ok(())
}
