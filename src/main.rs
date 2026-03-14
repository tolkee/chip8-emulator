mod emulator;
mod roms;
mod display;

use emulator::Emulator;
use emulator::EmulatorError;

fn main() {
    let mut emulator = Emulator::new();
    let rom_file_path: &str = roms::CHIP8_LOGO; 


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
    }

    emulator.run()
}   