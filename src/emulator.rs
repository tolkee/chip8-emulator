use std::fs;
use std::io;
use minifb::{Scale};
use rand::Rng;

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
    pub fn new() -> Self {
        Emulator {
            memory: [0; MAX_MEMORY],
            pc: ROM_START,
            i: 0,
            v: [0; 16],
            display: Display::new("chip8", 64, 32, Scale::X16),
            stack: [0; 16],
            sp: 0
        }
    }

    pub fn run(&mut self) -> Result<(), EmulatorError> {
        println!("[emulator] Running programm...");

        loop {
            let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16);
            let nibble_1 = (opcode & 0xF000) >> 12;
            let x = (opcode & 0x0F00) >> 8;
            let y = (opcode & 0x00F0) >> 4;
            let n = opcode & 0x000F;
            let nnn = opcode & 0x0FFF;
            let kk = (opcode & 0x00FF) as u8;  

            match nibble_1 {
                0x0 => {
                    if opcode == 0x00E0 {
                        self.display.clear();
                        self.pc += 2;
                    } else if opcode == 0x00EE {
                        self.pc = self.stack[self.sp as usize];
                        self.sp -= 1;
                    }
                },
                0x1 => self.pc = nnn,
                0x2 => {
                    self.sp += 1;
                    self.stack[self.sp as usize] = self.pc + 2;
                    self.pc = nnn;
                },
                0x3 => {
                    if self.v[x as usize] == kk {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },
                0x4 => {
                    if self.v[x as usize] != kk {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },
                0x5 => {
                    if n == 0 && self.v[x as usize] == self.v[y as usize] {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },
                0x6 => {
                    self.v[x as usize] = kk;
                    self.pc += 2;
                },
                0x7 => {
                    self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
                    self.pc += 2;
                },
                0x8 => {
                    match n {
                        0x0 => self.v[x as usize] = self.v[y as usize],
                        0x1 => self.v[x as usize] |= self.v[y as usize],
                        0x2 => self.v[x as usize] &= self.v[y as usize],
                        0x3 => self.v[x as usize] ^= self.v[y as usize],
                        0x4 => {
                            let result = self.v[x as usize] as u16 + self.v[y as usize] as u16;
                            let carry = if result > 0xFF {1} else {0}; 
                            self.v[x as usize] = result as u8;
                            self.v[0xF] = carry;           

                        },
                        0x5 => {
                            let not_borrow = if self.v[x as usize] >= self.v[y as usize] {1} else {0};
                            self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
                            self.v[0xF] = not_borrow;   
                        },
                        0x6 => {
                            let lsb = self.v[x as usize] & 1;
                            self.v[x as usize] >>= 1;
                            self.v[0xF] = lsb;

                        },
                        0x7 => {
                            let not_borrow = if self.v[y as usize] >= self.v[x as usize] {1} else {0}; 
                            self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);
                            self.v[0xF] = not_borrow;   
                        },
                        0xE => {
                            let msb = (self.v[x as usize] & 0x80) >> 7;
                            self.v[x as usize] <<= 1;
                            self.v[0xF] = msb;
                        },
                        _ => return Err(EmulatorError::InvalidOpCode(opcode))
                    };

                    self.pc += 2;
                },
                0x9 => {
                    if n == 0 && self.v[x as usize] != self.v[y as usize] {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },
                0xA => {
                    self.i = nnn;
                    self.pc += 2;
                },
                0xB => {
                    self.pc = nnn.wrapping_add(self.v[0] as u16);
                },
                0xC => {
                    let random_byte: u8 = rand::thread_rng().r#gen();

                    self.v[x as usize] = random_byte & kk;

                    self.pc += 2;
                },
                0xD => {
                    let bytes_to_draw: &[u8] = &self.memory[(self.i as usize)..((self.i as usize) + n as usize)];
                    let vx: u8 = self.v[x as usize];
                    let vy = self.v[y as usize];

                    let is_collisions = self.display.draw(bytes_to_draw, vx, vy);
                    self.v[0xF] = if is_collisions {1} else {0};

                    self.pc += 2;
                },
                0xE => {
                    match kk {
                        0x9E => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        0xA1 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        _ => return Err(EmulatorError::InvalidOpCode(opcode))
                    }
                },
                0xF => {
                    match kk {
                        0x07 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        0x0A => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        0x15 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        0x18 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        0x1E => {
                            self.i = self.i.wrapping_add(self.v[x as usize] as u16); 
                        },
                        0x29 => return Err(EmulatorError::NotImplementedOpCode(opcode)),
                        0x33 => {
                            let hundreds = self.v[x as usize] / 100;
                            let tens = (self.v[x as usize]/ 10) % 10;
                            let units = self.v[x as usize] % 10;

                            self.memory[self.i as usize] = hundreds;
                            self.memory[(self.i + 1) as usize] = tens;
                            self.memory[(self.i + 2) as usize] = units;
                        },
                        0x55 => {
                            for index in 0..=x {
                                self.memory[(self.i+index) as usize] = self.v[index as usize]
                            }
                        },
                        0x65 => {
                            for index in 0..=x {
                                self.v[index as usize] = self.memory[(self.i + index)  as usize]
                            }
                        },
                        _ => return Err(EmulatorError::InvalidOpCode(opcode))
                    };

                    self.pc += 2;
                },
                _ => return Err(EmulatorError::InvalidOpCode(opcode))
            }

            self.display.update();

            if self.pc >= (ROM_END - 1) {
                return Ok(());
            }
        }
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
}

pub enum EmulatorError {
    RomTooBig(usize),
    RomReadError(io::Error),
    InvalidOpCode(u16),
    NotImplementedOpCode(u16),
}