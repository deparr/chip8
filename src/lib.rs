use std::fmt::Display;

use rand::rngs::ThreadRng;
use rand::Rng;

const MEM_SIZE: usize = 4096;
const PROG_OFFSET: usize = 512;
const DISP_OFFSET: usize = MEM_SIZE - 256;
const INT_OFFSET: usize = DISP_OFFSET - 96;

const GFX_SIZE: usize = 64 * 32;

const FLAG_REG: usize = 15; // 0x0f

const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

type RegId = usize;
type Addr = usize;

#[derive(Debug)]
pub struct Cpu {
    regs: Vec<u8>,
    i: usize,
    pc: usize,
    sp: usize,
}

impl Cpu {
    fn new() -> Self {
        return Cpu {
            regs: vec![0; 16],
            i: 0,
            pc: PROG_OFFSET,
            sp: INT_OFFSET,
        };
    }
}

#[derive(Debug)]
enum OpCode {
    NativeCall(Addr),       // 0NNN
    DispClear,              // 00E0
    Ret,                    // 00EE
    Jmp(Addr),              // 1NNN
    Call(Addr),             // 2NNN
    ImEq(RegId, u8),        // 3XNN
    ImNeq(RegId, u8),       // 4XNN
    RREq(RegId, RegId),     // 5XY0
    IRMov(RegId, u8),       // 6XNN
    IRAdd(RegId, u8),       // 7XNN
    RRMov(RegId, RegId),    // 8XY0
    RROr(RegId, RegId),     // 8XY1
    RRAnd(RegId, RegId),    // 8XY2
    RRXor(RegId, RegId),    // 8XY3
    RRAdd(RegId, RegId),    // 8XY4
    RRSub(RegId, RegId),    // 8XY5
    RRShr(RegId),           // 8XY6
    RRSub2(RegId, RegId),   // 8XY7
    RRShl(RegId),           // 8XYE
    RRNeq(RegId, RegId),    // 9XY0
    Index(Addr),            // ANNN
    JmpAdd(Addr),           // BNNN
    Rand(RegId, u8),        // CXNN
    Draw(RegId, RegId, u8), // DXYN
    KeyEq(RegId),           // EX9E
    KeyNeq(RegId),          // EXA1
    DelayGet(RegId),        // FX07
    KeyWait(RegId),         // FX0A
    DelaySet(RegId),        // FX15
    SoundSet(RegId),        // FX18
    IncIndex(RegId),        // FX1E
    SpriteAddr(RegId),      // FX29
    BCD(RegId),             // FX33
    RegDump(RegId),         // FX55
    RegLoad(RegId),         // FX65
    Halt,                   // FFFF
    Invalid,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OpCode::*;
        match self {
            NativeCall(addr) => write!(f, "NativeCall@x{addr:04x}"),
            DispClear => write!(f, "DispClear"),
            Ret => write!(f, "Ret"),
            Jmp(addr) => write!(f, "Jmp@x{addr:04x}"),
            Call(addr) => write!(f, "Call@x{addr:04x}"),
            ImEq(vx, vi) => write!(f, "ImEq v{vx:X},{vi:02x}"),
            ImNeq(vx, vi) => write!(f, "ImNeq v{vx:X},{vi:02x}"),
            RREq(vx, vy) => write!(f, "RREq v{vx:X},v{vy:X}"),
            IRMov(vx, vi) => write!(f, "IRMov v{vx:X},{vi:02x}"),
            IRAdd(vx, vi) => write!(f, "IRAdd v{vx:X},{vi:02x}"),
            RRMov(vx, vy) => write!(f, "RRMov v{vx:X},v{vy:X}"),
            RROr(vx, vy) => write!(f, "RROr v{vx:X},v{vy:X}"),
            RRAnd(vx, vy) => write!(f, "RRAnd v{vx:X},v{vy:X}"),
            RRXor(vx, vy) => write!(f, "RRXor v{vx:X},v{vy:X}"),
            RRAdd(vx, vy) => write!(f, "RRAdd v{vx:X},v{vy:X}"),
            RRSub(vx, vy) => write!(f, "RRSub v{vx:X},v{vy:X}"),
            RRShr(vx) => write!(f, "RRShr v{vx:X}"),
            RRSub2(vx, vy) => write!(f, "RRSub2 v{vx:X},v{vy:X}"),
            RRShl(vx) => write!(f, "RRShl v{vx:X}"),
            RRNeq(vx, vy) => write!(f, "RRNeq v{vx:X},v{vy:X}"),
            Index(addr) => write!(f, "Index@x{addr:04x}"),
            JmpAdd(addr) => write!(f, "JmpAdd@x{addr:04x}"),
            Rand(vx, vi) => write!(f, "Rand v{vx:X},{vi:02x}"),
            Draw(vx, vy, n) => write!(f, "Draw v{vx:X},v{vy:X},{n}"),
            KeyEq(vx) => write!(f, "KeyEq v{vx:X}"),
            KeyNeq(vx) => write!(f, "KeyNeq v{vx:X}"),
            DelayGet(vx) => write!(f, "DelayGet v{vx:X}"),
            KeyWait(vx) => write!(f, "KeyWait v{vx:X}"),
            DelaySet(vx) => write!(f, "DelaySet v{vx:X}"),
            SoundSet(vx) => write!(f, "SoundSet v{vx:X}"),
            IncIndex(vx) => write!(f, "IncIndex v{vx:X}"),
            SpriteAddr(vx) => write!(f, "SpriteAddr v{vx:X}"),
            BCD(vx) => write!(f, "BCD v{vx:X}"),
            RegDump(vx) => write!(f, "RegDump v{vx:X}"),
            RegLoad(vx) => write!(f, "RegLoad v{vx:X}"),
            Halt => write!(f, "Halt"),
            Invalid => write!(f, "Invalid"),
        }
    }
}

#[derive(PartialEq)]
pub enum StepMode {
    Cycle,
    Debug,
}

pub struct Chip8 {
    cpu: Cpu,
    mem: Vec<u8>,
    pub gfx: Vec<u8>,
    keys: u16,
    pub cycles: u32,
    delay_timer: u8,
    sound_timer: u8,
    pub draw: bool,
    pub step_mode: StepMode,
    rng: ThreadRng,
    pub running: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut comp = Chip8 {
            cpu: Cpu::new(),
            mem: vec![0; MEM_SIZE],
            gfx: vec![0; GFX_SIZE],
            keys: 0,
            cycles: 0,
            delay_timer: 0,
            sound_timer: 0,
            draw: true,
            step_mode: StepMode::Cycle,
            rng: rand::thread_rng(),
            running: true,
        };

        comp.mem[0..80].copy_from_slice(&CHIP8_FONTSET);

        comp
    }

    pub fn with_mode(mut self, mode: StepMode) -> Self {
        self.step_mode = mode;
        self
    }

    pub fn load(&mut self, prog: &[u8]) -> Result<(), String> {
        if prog.len() >= INT_OFFSET - PROG_OFFSET {
            return Err(format!(
                "Program len({}) exceeds memory len({})",
                prog.len(),
                INT_OFFSET - PROG_OFFSET
            ));
        }

        self.mem[PROG_OFFSET..PROG_OFFSET + prog.len()].copy_from_slice(prog);

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), String> {
        let pc = self.cpu.pc;
        let opcode_num = match (self.mem.get(pc), self.mem.get(pc + 1)) {
            (Some(a), Some(b)) => (*a as u16) << 8 | *b as u16,
            _ => return Err(format!("An opcode byte was None at 0x{:04x}", pc)),
        };

        let opcode = self.decode(opcode_num);
        let mut skip = false;
        let mut next_pc = pc + 2;
        use OpCode::*;

        if self.step_mode == StepMode::Debug {
            // println!("{:?}", self.cpu);
            println!("pc: 0x{:04x}", pc);
            println!("opcode: 0x{:04x} ({})", opcode_num, opcode);
            println!("cycles: 0x{:08x}", self.cycles);
            // println!("keys: 0x{:04x}", self.keys);
        }

        match opcode {
            NativeCall(addr) => {
                self.mem[self.cpu.sp] = (next_pc & 0xf) as u8;
                self.mem[self.cpu.sp + 1] = ((next_pc >> 8) & 0xf) as u8;
                self.cpu.sp += 2;
                next_pc = addr;
            }
            DispClear => {
                for i in 0..self.gfx.len() {
                    self.gfx[i] = 0;
                }
                self.draw = true;
            }
            Ret => {
                self.cpu.sp -= 2;
                next_pc = self.mem[self.cpu.sp] as Addr;
                next_pc = next_pc << 8 | self.mem[self.cpu.sp + 1] as Addr;
                // println!("retting to 0x{:04x}", next_pc);
            }
            Jmp(addr) => {
                next_pc = addr as Addr;
            }
            Call(addr) => {
                self.mem[self.cpu.sp] = (next_pc >> 8) as u8;
                self.mem[self.cpu.sp + 1] = (next_pc & 0xff) as u8;
                self.cpu.sp += 2;
                // next_pc = (self.mem[addr + 1] as Addr) << 8 | self.mem[addr] as Addr;
                // next_pc = (self.mem[addr] as Addr) << 8 | (self.mem[addr + 1] as Addr)
                next_pc = addr;
                // println!("calling to 0x{:04x}", next_pc);
            }
            ImEq(reg, val) => {
                skip = self.cpu.regs[reg] == val;
            }
            ImNeq(reg, val) => {
                skip = self.cpu.regs[reg] != val;
            }
            RREq(ra, rb) => {
                skip = self.cpu.regs[ra] == self.cpu.regs[rb];
            }
            RRNeq(ra, rb) => {
                skip = self.cpu.regs[ra] != self.cpu.regs[rb];
            }
            IRMov(reg, val) => {
                self.cpu.regs[reg] = val;
            }
            // does not update carry
            IRAdd(reg, val) => {
                let (res, _) = self.cpu.regs[reg].overflowing_add(val);
                self.cpu.regs[reg] = res;
            }
            RRMov(ra, rb) => {
                self.cpu.regs[ra] = self.cpu.regs[rb];
            }
            RROr(ra, rb) => self.cpu.regs[ra] |= self.cpu.regs[rb],
            RRAnd(ra, rb) => self.cpu.regs[ra] &= self.cpu.regs[rb],
            RRXor(ra, rb) => self.cpu.regs[ra] ^= self.cpu.regs[rb],
            RRAdd(ra, rb) => {
                let (res, of) = self.cpu.regs[ra].overflowing_add(self.cpu.regs[rb]);
                self.cpu.regs[FLAG_REG] = if of { 1 } else { 0 };
                self.cpu.regs[ra] = res;
            }
            RRSub(ra, rb) => {
                let (res, of) = self.cpu.regs[ra].overflowing_sub(self.cpu.regs[rb]);
                self.cpu.regs[FLAG_REG] = if of { 0 } else { 1 };
                self.cpu.regs[ra] = res;
            }
            RRSub2(ra, rb) => {
                let (res, of) = self.cpu.regs[rb].overflowing_sub(self.cpu.regs[ra]);
                self.cpu.regs[FLAG_REG] = if of { 0 } else { 1 };
                self.cpu.regs[ra] = res;
            }
            RRShr(ra) => {
                let ra_val = self.cpu.regs[ra];
                self.cpu.regs[FLAG_REG] = ra_val & 0x1;
                self.cpu.regs[ra] = ra_val >> 1;
            }
            RRShl(ra) => {
                let ra_val = self.cpu.regs[ra];
                self.cpu.regs[FLAG_REG] = (ra_val & 0x80) >> 7;
                self.cpu.regs[ra] = ra_val << 1;
            }
            Index(addr) => {
                self.cpu.i = addr;
            }
            JmpAdd(addr) => {
                next_pc = self.cpu.regs[0] as Addr + addr;
            }
            Rand(reg, val) => {
                let rand_val: u8 = self.rng.gen();
                self.cpu.regs[reg] = rand_val & val;
            }
            Draw(vx, vy, n) => {
                self.cpu.regs[0xf] = 0;
                let vx = self.cpu.regs[vx] as usize;
                let vy = self.cpu.regs[vy] as usize;
                let n = n as usize;
                for y in 0..n {
                    let pixel = self.mem[self.cpu.i + y as usize];
                    for x in 0..8 {
                        if pixel & (0x80 >> x) != 0 {
                            let idx = vx + x + (y + vy) * 64;
                            if self.gfx[idx] == 1 {
                                self.cpu.regs[0xf] = 1;
                            }

                            self.gfx[idx] ^= 1
                        }
                    }
                }
                self.draw = true;
            }
            KeyEq(vx) => {
                skip = (self.keys >> self.cpu.regs[vx] & 1) == 1;
            }
            KeyNeq(vx) => {
                skip = (self.keys >> self.cpu.regs[vx] & 1) == 0;
            }
            DelayGet(vx) => {
                self.cpu.regs[vx] = self.delay_timer;
            }
            DelaySet(vx) => {
                self.delay_timer = self.cpu.regs[vx];
            }
            SoundSet(vx) => {
                self.sound_timer = self.cpu.regs[vx];
            }
            KeyWait(vx) => {
                let mut got_key = false;
                for i in 0..16 {
                    if (self.keys & (1 << i)) > 0 {
                        self.cpu.regs[vx] = i;
                        got_key = true;
                        break;
                    }
                }

                if !got_key {
                    next_pc = pc;
                }
            }
            IncIndex(vx) => {
                self.cpu.i += self.cpu.regs[vx] as Addr;
            }
            SpriteAddr(vx) => {
                self.cpu.i = (self.cpu.regs[vx] * 5) as Addr;
            }
            BCD(vx) => {
                let vx = self.cpu.regs[vx];
                let h = vx / 100;
                let t = (vx % 100) / 10;
                let o = vx % 10;

                self.mem[self.cpu.i] = h;
                self.mem[self.cpu.i + 1] = t;
                self.mem[self.cpu.i + 2] = o;
            }
            RegDump(vx) => {
                for x in 0..=vx {
                    self.mem[self.cpu.i + x] = self.cpu.regs[vx];
                }
                self.cpu.i += vx + 1;
            }
            RegLoad(vx) => {
                for x in 0..=vx {
                    self.cpu.regs[vx] = self.mem[self.cpu.i + x]
                }
                self.cpu.i += vx + 1;
            }
            Halt => self.running = false,
            Invalid => {
                return Err(format!(
                    "Invalid opcode: 0x{:04x} at pc: 0x{:04x}",
                    opcode_num, pc
                ));
            }
        }

        if skip {
            next_pc += 2;
        }

        if self.step_mode == StepMode::Debug {
            println!("next_pc: {:04x}\n", next_pc);
        }

        self.cpu.pc = next_pc;
        self.cycles += 1;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("TODO: BEEP");
            }

            self.sound_timer -= 1;
        }

        Ok(())
    }

    fn decode(&self, opcode: u16) -> OpCode {
        use OpCode::*;
        let icode = opcode >> 12;
        let ifun = opcode & 0x0f;
        let addr = (opcode & 0x0fff) as Addr;
        let vx = ((opcode >> 8) & 0x0f) as RegId;
        let vy = ((opcode >> 4) & 0x0f) as RegId;
        let vi = (opcode & 0xff) as u8;
        match icode {
            0x0 => match vi {
                0xee => Ret,
                0xe0 => DispClear,
                _ => NativeCall(addr),
            },
            0x1 => Jmp(addr),
            0x2 => Call(addr),
            0x3 => ImEq(vx, vi),
            0x4 => ImNeq(vx, vi),
            0x5 => RREq(vx, vy), // note: ifun ??
            0x6 => IRMov(vx, vi),
            0x7 => IRAdd(vx, vi),
            0x8 => match ifun {
                0x0 => RRMov(vx, vy),
                0x1 => RROr(vx, vy),
                0x2 => RRAnd(vx, vy),
                0x3 => RRXor(vx, vy),
                0x4 => RRAdd(vx, vy),
                0x5 => RRSub(vx, vy),
                0x6 => RRShr(vx),
                0x7 => RRSub2(vx, vy),
                0xe => RRShl(vx),
                _ => Invalid,
            },
            0x9 => RRNeq(vx, vy),
            0xa => Index(addr),
            0xb => JmpAdd(addr),
            0xc => Rand(vx, vi),
            0xd => Draw(vx, vy, ifun as u8),
            0xe => match vi {
                0x9e => KeyEq(vx),
                0xa1 => KeyNeq(vx),
                _ => Invalid,
            },
            0xf => match vi {
                0x07 => DelayGet(vx),
                0x0A => KeyWait(vx),
                0x15 => DelaySet(vx),
                0x18 => SoundSet(vx),
                0x1E => IncIndex(vx),
                0x29 => SpriteAddr(vx),
                0x33 => BCD(vx),
                0x55 => RegDump(vx),
                0x65 => RegLoad(vx),
                0xff => Halt,
                _ => Invalid,
            },
            _ => Invalid,
        }
    }

    pub fn key_down(&mut self, key: usize) {
        self.keys |= 1 << key;
    }

    pub fn key_up(&mut self, key: usize) {
        self.keys &= !(1 << key);
    }

    pub fn dec_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
