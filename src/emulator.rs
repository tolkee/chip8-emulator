use std::fs;
use rand::Rng;

use crate::errors::EmulatorError;
use crate::display::Display;

// Here some info about memory allocation from chip8 spec :
// 4KB (4,096 bytes) of RAM, from location 0x000 (0) to 0xFFF (4095)
// - 0x000 to 0x1FF (first 512 bytes) is where the original interpreter was located, and should not be used by programs.
// - most of the programs start at 0x200 (512)

const MAX_MEMORY: usize = 4096;
const ROM_START: u16 = 0x200;
const ROM_END: u16 = 0xFFF;
const ROM_MAX_LEN: usize = (ROM_END - ROM_START) as usize;

pub struct Emulator {
    memory: [u8; MAX_MEMORY],
    pc: u16,
    i: u16,
    v: [u8; 16], // all purpose register
    display: Display,
    stack: [u16; 16],
    sp: u8,
}

impl Emulator {
    pub fn new(display: Display) -> Self {
        Emulator {
            memory: [0; MAX_MEMORY],
            pc: ROM_START,
            i: 0,
            v: [0; 16],
            sp: 0,
            stack: [0; 16],
            display,
        }
    }

    pub fn run(&mut self) -> Result<(), EmulatorError> {
        println!("[emulator] Running programm...");

        loop {
            self.cpu_cycle()?;
            self.display.update();

            if self.pc >= (ROM_END - 1) {
                return Ok(());
            }
        }
    }

    fn cpu_cycle(&mut self) -> Result<(), EmulatorError> {
        let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16);
        let nibble_1 = (opcode & 0xF000) >> 12;
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let n = opcode & 0x000F;
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;

        match nibble_1 {
            0x0 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => return Err(EmulatorError::InvalidOpCode(opcode)),
            }
            0x1 => self.op_1nnn(nnn),
            0x2 => self.op_2nnn(nnn),
            0x3 => self.op_3xkk(x, kk),
            0x4 => self.op_4xkk(x, kk),
            0x5 => self.op_5xy0(x, y, n),
            0x6 => self.op_6xkk(x, kk),
            0x7 => self.op_7xkk(x, kk),
            0x8 => match n {
                0x0 => self.op_8xy0(x, y),
                0x1 => self.op_8xy1(x, y),
                0x2 => self.op_8xy2(x, y),
                0x3 => self.op_8xy3(x, y),
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                0x6 => self.op_8xy6(x),
                0x7 => self.op_8xy7(x, y),
                0xE => self.op_8xy_e(x),
                _ => return Err(EmulatorError::InvalidOpCode(opcode)),
            },
            0x9 => self.op_9xy0(x, y, n),
            0xA => self.op_annn(nnn),
            0xB => self.op_bnnn(nnn),
            0xC => self.op_cxkk(x, kk),
            0xD => self.op_dxyn(x, y, n),
            0xE => match kk {
                0x9E => self.op_ex9e(x),
                0xA1 => self.op_exa1(x),
                _ => return Err(EmulatorError::InvalidOpCode(opcode)),
            },
            0xF => match kk {
                0x07 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                0x0A => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                0x15 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                0x18 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                0x1E => self.op_fx1e(x),
                0x29 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                0x33 => self.op_fx33(x),
                0x55 => self.op_fx55(x),
                0x65 => self.op_fx65(x),
                _ => return Err(EmulatorError::InvalidOpCode(opcode)),
            },
            _ => return Err(EmulatorError::InvalidOpCode(opcode)),
        }

        Ok(())
    }

    pub fn load_rom_file(&mut self, path: &str) -> Result<(), EmulatorError> {
        let rom: Vec<u8> = fs::read(path).map_err(|io_error| EmulatorError::RomReadError(io_error))?;
        
        self.load_rom(&rom)
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), EmulatorError> {
        // wiping the prog allocated memory (in case another rom was running before)
        self.memory[(ROM_START as usize)..=(ROM_END as usize)].fill(0);


        if rom.len() > ROM_MAX_LEN {
            return Err(EmulatorError::RomTooBig(ROM_MAX_LEN - rom.len()));
        }

        self.memory[ROM_START as usize..ROM_START as usize + rom.len()].copy_from_slice(&rom);

        Ok(())
    }

    fn op_00e0(&mut self) {
        self.display.clear();
        self.pc += 2;
    }

    fn op_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc + 2;
        self.pc = nnn;
    }

    fn op_3xkk(&mut self, x: u16, kk: u8) {
        if self.v[x as usize] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_4xkk(&mut self, x: u16, kk: u8) {
        if self.v[x as usize] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x: u16, y: u16, n: u16) {
        if n == 0 && self.v[x as usize] == self.v[y as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_6xkk(&mut self, x: u16, kk: u8) {
        self.v[x as usize] = kk;
        self.pc += 2;
    }

    fn op_7xkk(&mut self, x: u16, kk: u8) {
        self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
        self.pc += 2;
    }

    fn op_8xy0(&mut self, x: u16, y: u16) {
        self.v[x as usize] = self.v[y as usize];
        self.pc += 2;
    }

    fn op_8xy1(&mut self, x: u16, y: u16) {
        self.v[x as usize] |= self.v[y as usize];
        self.pc += 2;
    }

    fn op_8xy2(&mut self, x: u16, y: u16) {
        self.v[x as usize] &= self.v[y as usize];
        self.pc += 2;
    }

    fn op_8xy3(&mut self, x: u16, y: u16) {
        self.v[x as usize] ^= self.v[y as usize];
        self.pc += 2;
    }

    fn op_8xy4(&mut self, x: u16, y: u16) {
        let result = self.v[x as usize] as u16 + self.v[y as usize] as u16;
        let carry = if result > 0xFF { 1 } else { 0 };
        self.v[x as usize] = result as u8;
        self.v[0xF] = carry;
        self.pc += 2;
    }

    fn op_8xy5(&mut self, x: u16, y: u16) {
        let not_borrow = if self.v[x as usize] >= self.v[y as usize] { 1 } else { 0 };
        self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
        self.v[0xF] = not_borrow;
        self.pc += 2;
    }

    fn op_8xy6(&mut self, x: u16) {
        let lsb = self.v[x as usize] & 1;
        self.v[x as usize] >>= 1;
        self.v[0xF] = lsb;
        self.pc += 2;
    }

    fn op_8xy7(&mut self, x: u16, y: u16) {
        let not_borrow = if self.v[y as usize] >= self.v[x as usize] { 1 } else { 0 };
        self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);
        self.v[0xF] = not_borrow;
        self.pc += 2;
    }

    fn op_8xy_e(&mut self, x: u16) {
        let msb = (self.v[x as usize] & 0x80) >> 7;
        self.v[x as usize] <<= 1;
        self.v[0xF] = msb;
        self.pc += 2;
    }

    fn op_9xy0(&mut self, x: u16, y: u16, n: u16) {
        if n == 0 && self.v[x as usize] != self.v[y as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
        self.pc += 2;
    }

    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn.wrapping_add(self.v[0] as u16);
    }

    fn op_cxkk(&mut self, x: u16, kk: u8) {
        let random_byte: u8 = rand::thread_rng().r#gen();
        self.v[x as usize] = random_byte & kk;
        self.pc += 2;
    }

    fn op_dxyn(&mut self, x: u16, y: u16, n: u16) {
        let bytes_to_draw: &[u8] = &self.memory[(self.i as usize)..((self.i as usize) + n as usize)];
        let vx: u8 = self.v[x as usize];
        let vy = self.v[y as usize];
        let is_collisions = self.display.draw(bytes_to_draw, vx, vy);
        self.v[0xF] = if is_collisions { 1 } else { 0 };
        self.pc += 2;
    }

    fn op_ex9e(&mut self, x: u16) {
        if self.display.is_key_down(self.v[x as usize]) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_exa1(&mut self, x: u16) {
        if !self.display.is_key_down(self.v[x as usize]) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_fx1e(&mut self, x: u16) {
        self.i = self.i.wrapping_add(self.v[x as usize] as u16);
        self.pc += 2;
    }

    fn op_fx33(&mut self, x: u16) {
        let hundreds = self.v[x as usize] / 100;
        let tens = (self.v[x as usize] / 10) % 10;
        let units = self.v[x as usize] % 10;
        self.memory[self.i as usize] = hundreds;
        self.memory[(self.i + 1) as usize] = tens;
        self.memory[(self.i + 2) as usize] = units;
        self.pc += 2;
    }

    fn op_fx55(&mut self, x: u16) {
        for index in 0..=x {
            self.memory[(self.i + index) as usize] = self.v[index as usize];
        }
        self.pc += 2;
    }

    fn op_fx65(&mut self, x: u16) {
        for index in 0..=x {
            self.v[index as usize] = self.memory[(self.i + index) as usize];
        }
        self.pc += 2;
    }
}
