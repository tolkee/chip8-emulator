use std::fs;
use std::io;
use minifb::{Scale};

use crate::display::Display;

// Here some info about memory allocation from chip8 spec :
// 4KB (4,096 bytes) of RAM, from location 0x000 (0) to 0xFFF (4095)
// - 0x000 to 0x1FF (first 512 bytes) is where the original interpreter was located, and should not be used by programs.
// - most of the programs start at 0x200 (512)

const MAX_MEMORY: usize = 4096;
const ROM_START: u16 = 0x200;
const ROM_END: u16 = 0xFFF;

pub struct Emulator {
    memory: [u8; MAX_MEMORY],
    pc: u16,
    i: u16,
    v: [u8; 16], // all purpose register
    display: Display,
    vf: u8,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            memory: [0; MAX_MEMORY],
            pc: ROM_START,
            i: 0,
            v: [0; 16],
            display: Display::new("chip8", 64, 32, Scale::X32),
            vf: 0,
        }
    }

    pub fn run(&mut self) {
        println!("[emulator] Running programm...");

        loop {
            let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16);
            let nibble_1 = (opcode & 0xF000) >> 12;
            let nibble_2 = (opcode & 0x0F00) >> 8;
            let nibble_3 = (opcode & 0x00F0) >> 4;
            let nibble_4 = opcode & 0x000F;
            let nnn = opcode & 0x0FFF;
            let kk = (opcode & 0x00FF) as u8;
            

            match nibble_1 {
                0x0 => {
                    if opcode == 0x00E0 {
                        self.display.clear();
                        self.pc += 2;
                    } else if opcode == 0x00EE {
                        ()
                    } else {
                        ()
                    }

                },
                0x1 => self.pc = nnn,
                0x2 => (),
                0x3 => (),
                0x4 => (),
                0x5 => (),
                0x6 => {
                    self.v[nibble_2 as usize] = kk;
                    self.pc += 2;
                },
                0x7 => (),
                0x8 => (),
                0x9 => (),
                0xA => {
                    self.i = nnn;
                    self.pc += 2;
                },
                0xB => (),
                0xC => (),
                0xD => {
                    let bytes_to_draw: &[u8] = &self.memory[(self.i as usize)..((self.i as usize) + nibble_4 as usize)];
                    let vx: u8 = self.v[nibble_2 as usize];
                    let vy = self.v[nibble_3 as usize];

                    let is_collisions = self.display.draw(bytes_to_draw.to_vec(), vx, vy);
                    self.vf = if is_collisions {1} else {0};

                    self.pc += 2;
                },
                0xE => (),
                0xF => (),
                _ => ()
            }

            self.display.update();

            if self.pc >= (ROM_END - 1) {
                break;
            }
        }
    }

    pub fn load_rom_file(&mut self, path: &str) -> Result<(), EmulatorError> {
        let rom: Vec<u8> = fs::read(path).map_err(|io_error| EmulatorError::RomReadError(io_error))?;
        
        self.load_rom(rom)
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) -> Result<(), EmulatorError> {
        // wiping the prog allocated memory (in case another rom was running before)
        self.memory[(ROM_START as usize)..=(ROM_END as usize)].fill(0);

        let mut memory_offset = ROM_START;

        for byte in rom {
            if memory_offset > ROM_END {
                let bytes_overflow = memory_offset - ROM_END;
                return Err(EmulatorError::RomTooBig(bytes_overflow));
            }

            self.memory[memory_offset as usize] = byte;
            memory_offset += 1;
        }

        Ok({})
    }
}

pub enum EmulatorError {
    RomTooBig(u16),
    RomReadError(io::Error)
}