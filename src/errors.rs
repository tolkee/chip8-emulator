use std::io;

pub enum EmulatorError {
    RomTooBig(usize),
    RomReadError(io::Error),
    InvalidOpCode(u16),
    NotImplementedOpCode(u16),
}