use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result};
pub enum Chip8Error {
    ProgramTooLarge,
}

impl Chip8Error {
    fn message(&self) -> &str {
        match self {
            Chip8Error::ProgramTooLarge => "The program size is too large",
        }
    }
}

impl Error for Chip8Error {}

impl Display for Chip8Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message())
    }
}

impl Debug for Chip8Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message())
    }
}
