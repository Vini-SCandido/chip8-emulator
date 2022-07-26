use std::usize;

use rand::random;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st:0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(&data);
    }
    
    fn push(&mut self, val: u16) {
        if self.sp == 15 {
            eprintln!("\nERROR:\n    the stack pointer must not exceed the stack size\n");
            std::process::exit(0);
        }
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        if self.sp == 0 {
            eprintln!("\nERROR:\n    the stack pointer must be positive or zero\n");
            std::process::exit(0);
        }

        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        if self.pc as usize == self.ram.len() {
            eprintln!("\nERROR:\n    the program tried to read a value beyond the RAM size\n");
            std::process::exit(0);
        }
        let fst_byte = self.ram[self.pc as usize] as u16;
        let snd_byte = self.ram[(self.pc + 1) as usize] as u16;

        let op = (fst_byte << 8) | snd_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }
    
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn key_pressed(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    fn execute(&mut self, op: u16) {
        let hex1 = (op & 0xF000) >> 12;
        let hex2 = (op & 0x0F00) >> 8;
        let hex3 = (op & 0x00F0) >> 4;
        let hex4 = op & 0x000F;

        match (hex1, hex2, hex3, hex4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            },
            (1, _, _, _) => {
                self.pc = op & 0x0FFF;
            },
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0x0FFF;
            },
            (3, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                if self.v_reg[hex2 as usize] == nn {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                if self.v_reg[hex2 as usize] != nn {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => {
                if self.v_reg[hex2 as usize] == self.v_reg[hex3 as usize] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                self.v_reg[hex2 as usize] = nn;
            },
            (7, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                self.v_reg[hex2 as usize] = self.v_reg[hex2 as usize].wrapping_add(nn);
            },
            (8, _, _, 0) => {
                self.v_reg[hex2 as usize] = self.v_reg[hex3 as usize];
            },
            (8, _, _, 1) => {
                self.v_reg[hex2 as usize] |= self.v_reg[hex3 as usize];
            },
            (8, _, _, 2) => {
                self.v_reg[hex2 as usize] &= self.v_reg[hex3 as usize];
            },
            (8, _, _, 3) => {
                self.v_reg[hex2 as usize] ^= self.v_reg[hex3 as usize];
            },
            (8, _, _, 4) => {
                let x = hex2 as usize;
                let y = hex3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 5) => {
                let x = hex2 as usize;
                let y = hex3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 6) => {
                let dropped_bit = self.v_reg[hex2 as usize] & 1;
                self.v_reg[hex2 as usize] >>= 1;
                self.v_reg[0xF] = dropped_bit;
            },
            (8, _, _, 7) => {
                let x = hex2 as usize;
                let y = hex3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            (8, _, _, 0xE) => {
                let dropped_bit = (self.v_reg[hex2 as usize] >> 7) & 1;
                self.v_reg[hex2 as usize] <<= 1;
                self.v_reg[0xF] = dropped_bit;
            },
            (9, _, _, 0) => {
                if self.v_reg[hex2 as usize] != self.v_reg[hex3 as usize] {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            (0xC, _, _, _) => {
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[hex2 as usize] = rng & nn;
            },
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[hex2 as usize] as u16;
                let y_coord = self.v_reg[hex3 as usize] as u16;

                let num_rows = hex4;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b10000000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            (0xE, _, 9, 0xE) => {
                let vx = self.v_reg[hex2 as usize] as usize;
                let key = self.keys[vx];
                if key {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => {
                let vx = self.v_reg[hex2 as usize] as usize;
                let key = self.keys[vx];
                if !key {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => {
                self.v_reg[hex2 as usize] = self.dt;
            },
            (0xF, _, 0, 0xA) => {
                let x = hex2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 1, 5) => {
                self.dt = self.v_reg[hex2 as usize];
            },
            (0xF, _, 1, 8) => {
                self.st = self.v_reg[hex2 as usize];
            },
            (0xF, _, 1, 0xE) => {
                let vx = self.v_reg[hex2 as usize] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            (0xF, _, 2, 9) => {
                let c = self.v_reg[hex2 as usize] as u16;
                self.i_reg = c * 5;
            },
            (0xF, _, 3, 3) => {
                let vx = self.v_reg[hex2 as usize] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0).floor() as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            (0xF, _, 5, 5) => {
                let x = hex2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            },
            (0xF, _, 6, 5) => {
                let x = hex2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },
            (_, _, _, _) => {
                eprintln!("\nERROR:\n    an unknown operation was found during the program's execution\n");
                std::process::exit(0);
            }
        }
    }
}
