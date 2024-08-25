use rand::Rng;
use rand::rngs::ThreadRng;

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
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


type RegId = usize;
type Addr = usize;

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

enum OpCode {
    Call(Addr),              // 0NNN
    DispClear,              // 00E0
    Ret,                    // 00EE
    Jmp(Addr),              // 1NNN
    CallAt(Addr),       // 2NNN
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
    RRShr(RegId, RegId),    // 8XY6
    RRSub2(RegId, RegId),   // 8XY7
    RRShl(RegId, RegId),    // 8XYE
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
    Invalid,
}

pub struct Chip8 {
    cpu: Cpu,
    mem: Vec<u8>,
    gfx: Vec<u8>,
    keys: u16,
    delay_timer: u8,
    sound_timer: u8,
    pub draw: bool,
    rng: ThreadRng,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut comp = Chip8 {
            cpu: Cpu::new(),
            mem: vec![0; MEM_SIZE],
            gfx: vec![0; GFX_SIZE],
            keys: 0,
            delay_timer: 0,
            sound_timer: 0,
            draw: false,
            rng: rand::thread_rng()
        };

        // for i in 0..FONT_OFFSET {
        // }

        comp.mem.copy_from_slice(&CHIP8_FONTSET);

        comp
    }

    pub fn step(&mut self) -> Result<(), usize> {
        let pc = self.cpu.pc;
        let opcode = match (self.mem.get(pc), self.mem.get(pc + 1)) {
            (Some(a), Some(b)) => (*a as u16) << 8 | *b as u16,
            _ => return Err(1),
        };

        let opcode = self.decode(opcode);
        let mut skip = false;
        let mut next_pc = pc + 2;
        use OpCode::*;
        match opcode {
            Call(addr) => {
                self.mem[self.cpu.sp] = (next_pc & 0xf) as u8;
                self.mem[self.cpu.sp + 1] = ((next_pc >> 8) & 0xf) as u8;
                self.cpu.sp += 2;
                next_pc = addr;
            },
            DispClear => todo!("emit display clear"),
            Ret => {
                // TODO get both bytes
                next_pc = self.mem[self.cpu.sp] as Addr;
                // TODO handle bot of stack correctly
                self.cpu.sp -= 2;
            },
            Jmp(addr) => {
                next_pc = addr as Addr;
            },
            CallAt(addr) => {
                // TODO this should probably push ret addr
                self.mem[self.cpu.sp] = (next_pc & 0xf) as u8;
                self.mem[self.cpu.sp + 1] = ((next_pc >> 8) & 0xf) as u8;
                self.cpu.sp += 2;
                next_pc = self.mem[addr] as Addr | (self.mem[addr + 1] as Addr) << 8;
            },
            ImEq(reg, val) => {
                skip = self.cpu.regs[reg] == val;
            },
            ImNeq(reg, val) => {
                skip = self.cpu.regs[reg] != val;
            },
            RREq(ra, rb) => {
                skip = self.cpu.regs[ra] == self.cpu.regs[rb];
            },
            RRNeq(ra, rb) => {
                skip = self.cpu.regs[ra] != self.cpu.regs[rb];
            },
            IRMov(reg, val) => {
                self.cpu.regs[reg] = val;
            },
            // does not update carry
            IRAdd(reg, val) => {
                // TODO overflow?
                self.cpu.regs[reg] += val;
            },
            RRMov(ra, rb) => {
                self.cpu.regs[ra] = self.cpu.regs[rb];
            },
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
            },
            RRSub2(ra, rb) => {
                let (res, of) = self.cpu.regs[rb].overflowing_sub(self.cpu.regs[ra]);
                self.cpu.regs[FLAG_REG] = if of { 0 } else { 1 };
                self.cpu.regs[ra] = res;
            }
            RRShr(ra, _) => {
                let ra_val = self.cpu.regs[ra];
                self.cpu.regs[FLAG_REG] = ra_val & 0x1;
                self.cpu.regs[ra] = ra_val >> 1;
            },
            RRShl(ra, _) => {
                let ra_val = self.cpu.regs[ra];
                self.cpu.regs[FLAG_REG] = (ra_val & 0x80) >> 7;
                self.cpu.regs[ra] = ra_val << 1;
            },
            Index(addr) => {
                self.cpu.i = addr;
            },
            JmpAdd(addr) => {
                next_pc = self.cpu.regs[0] as Addr + addr;
            },
            Rand(reg, val) => {
                let rand_val: u8 = self.rng.gen();
                self.cpu.regs[reg] = rand_val & val;
            },
            Draw(ra, rb, n) => {
                todo!("opcode Draw()")
            },
            KeyEq(vx) => {
                skip = (self.keys >> self.cpu.regs[vx] & 1) == 1;
            },
            KeyNeq(vx) => {
                skip = (self.keys >> self.cpu.regs[vx] & 1) == 0;
            },
            DelayGet(vx) => {
                self.cpu.regs[vx] = self.delay_timer;
            },
            DelaySet(vx) => {
                self.delay_timer = self.cpu.regs[vx];
            },
            SoundSet(vx) => {
                self.sound_timer = self.cpu.regs[vx];
            },
            KeyWait(vx) => {
                todo!("opcode KeyWait()")
            },
            IncIndex(vx) => {
                self.cpu.i += self.cpu.regs[vx] as Addr;
            },
            SpriteAddr(vx) => {
                todo!("opcode SpriteAddr()")
            },
            BCD(vx) => {
                todo!("opcode BCD()")
            },
            RegDump(vx) => {
                for x in 0..=vx {
                    self.mem[self.cpu.i + x] = self.cpu.regs[vx];
                }
            },
            RegLoad(vx) => {
                for x in 0..=vx {
                   self.cpu.regs[vx] = self.mem[self.cpu.i + x]
                }
            }
            Invalid => {
                todo!("opcode Invalid() handle it properly")
            }
        }

        if skip {
            next_pc += 2;
        }

        self.cpu.pc = next_pc;


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
                _ => Call(addr),
            },
            0x1 => Jmp(addr),
            0x2 => CallAt(addr),
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
                0x6 => RRShr(vx, vy),
                0x7 => RRSub2(vx, vy),
                0xe => RRShl(vx, vy),
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
                _ => Invalid
            }
            _ => Invalid,
        }
    }
}
