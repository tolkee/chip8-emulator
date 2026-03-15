mod emulator;
mod roms;
mod display;
mod errors;

use emulator::Emulator;
use errors::EmulatorError;
use minifb::{Scale};

use crate::display::Display;


fn main() {
    let display = Display::new("chip8", 64, 32, Scale::X16);
    let mut emulator = Emulator::new(display);

    let rom_file_path: &str = roms::FLAGS; 


    match emulator.load_rom_file(rom_file_path) {
        Ok(_) => println!("[emulator] {} rom loaded", rom_file_path),
        Err(EmulatorError::RomTooBig(size)) => {
            println!("[emulator] Error: rom too big by {} bytes", size);
            std::process::exit(1);
        },
        Err(EmulatorError::RomReadError(io_error)) => {
            println!("[emulator] Error: {}", io_error);
            std::process::exit(1);
        },
        _ => ()
    }

    match emulator.run() {
        Err(EmulatorError::InvalidOpCode(opcode)) => {
            println!("[emulator] Error: Invalid OPcode {}", opcode);
            std::process::exit(1);
        },
        Err(EmulatorError::NotImplementedOpCode(opcode)) => {
            println!("[emulator] Error: Not implemented OPcode {}", opcode);
            std::process::exit(1);
        },

        _ => ()
    }
}   