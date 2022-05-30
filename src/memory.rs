use std::fmt::{Debug, Formatter};
use std::io::Read;
use std::{fs, io};

const ADDRESS_INTERPRETER_START: usize = 0x0;
const ADDRESS_PROGRAM_START: usize = 0x200;
const ADDRESS_MAX: usize = 0xFFF;

const INTERPRETER_MEMORY_SIZE: usize = ADDRESS_PROGRAM_START - ADDRESS_INTERPRETER_START;
const PROGRAM_MEMORY_SIZE: usize = ADDRESS_MAX - ADDRESS_PROGRAM_START + 1;
const MEMORY_SIZE: usize = INTERPRETER_MEMORY_SIZE + PROGRAM_MEMORY_SIZE;

const SPRITE_SIZE: u8 = 5;

type HexSprite = [u8; SPRITE_SIZE as usize];

const HEX_SPRITES: [HexSprite; 0x10] = [
    // Numerical 0-9
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    // Alpha A-F
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];

pub type Address = u16;

pub struct RAM {
    value: [u8; MEMORY_SIZE],
}

pub trait ProgramLoader {
    type Output;

    fn load_into_ram(self, ram: &mut [u8]) -> Self::Output;
}

impl ProgramLoader for &[u8] {
    type Output = ();

    fn load_into_ram(self, ram: &mut [u8]) {
        ram[..self.len()].copy_from_slice(self)
    }
}

impl ProgramLoader for fs::File {
    type Output = io::Result<usize>;

    fn load_into_ram(mut self, ram: &mut [u8]) -> Self::Output {
        let mut buf = Vec::new();

        let bytes = self.read_to_end(&mut buf)?;
        (&mut buf[..]).load_into_ram(ram);

        Ok(bytes)
    }
}

impl RAM {
    pub fn new() -> RAM {
        // Start with zeroed RAM
        let mut ram = RAM {
            value: [0; MEMORY_SIZE],
        };

        // Initialize the start of the RAM with the interpreter memory
        let mut write_start = ADDRESS_INTERPRETER_START;

        // Hexadecimal sprites have real addresses in interpreter memory
        for sprite in HEX_SPRITES {
            let write_end = write_start + sprite.len();

            ram.value[write_start..write_end].copy_from_slice(&sprite);
            write_start = write_end;
        }

        ram
    }

    pub fn program_memory(&self) -> &[u8] {
        &self.value[ADDRESS_PROGRAM_START..]
    }

    pub fn program_memory_mut(&mut self) -> &mut [u8] {
        &mut self.value[ADDRESS_PROGRAM_START..]
    }

    pub fn address(&self, address: Address) -> &[u8] {
        let address = address as usize;

        &self.value[address..]
    }

    pub fn address_mut(&mut self, address: Address) -> &mut [u8] {
        let address = address as usize;

        &mut self.value[address..]
    }

    pub fn get_instruction(&self, address: Address) -> &[u8] {
        let address = address as usize;

        &self.value[address..address + 2]
    }

    pub fn get_address_of_sprite(&self, value: u8) -> Address {
        if value > 0xF {
            panic!("Cannot provide address of sprite with value {:#X}", value);
        }

        5 * value as u16
    }

    pub fn get_sprite_at_address(&self, address: Address, bytes: u8) -> &[u8] {
        let address = address as usize;
        let bytes = bytes as usize;

        &self.value[address..address + bytes]
    }

    pub fn load_program<T, U>(&mut self, loader: T) -> U
    where
        T: ProgramLoader<Output = U>,
    {
        loader.load_into_ram(self.program_memory_mut())
    }

    pub fn sprite_address(sprite_number: u8) -> Address {
        if sprite_number > 0xF {
            panic!("Invalid sprite number {}", sprite_number);
        }

        sprite_number as Address * SPRITE_SIZE as Address
    }
}

impl Debug for RAM {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let bytes_per_line = 10;

        let mut bytes_written = 0;

        for byte in self.value {
            if bytes_written % bytes_per_line == 0 {
                write!(f, "{:#05X}: ", bytes_written)?;
            }

            write!(f, "{:02X}", byte)?;
            bytes_written = bytes_written + 1;

            if bytes_written % bytes_per_line == 0 {
                writeln!(f, "")?;
            } else {
                write!(f, " ")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_memory() {
        let memory = RAM::new();

        assert_eq!(PROGRAM_MEMORY_SIZE, memory.program_memory().len())
    }

    #[test]
    fn load_into_memory() {
        let mut memory = RAM::new();

        // Loading a blank program should not affect anything
        let original_memory = memory.value.clone();
        memory.load_program(&[] as &[u8]);
        assert_eq!(&original_memory, &memory.value,);

        // Load a few instructions
        let program: [u8; 2] = [0x60, 0x50];
        memory.load_program(&program[..]);
        assert_eq!(program, memory.program_memory()[0..2]);
    }

    #[test]
    fn sprite_addressing() {
        let memory = RAM::new();

        for i in 0..0xF {
            let address = memory.get_address_of_sprite(i);
            let sprite = memory.get_sprite_at_address(address, 5);
            assert_eq!(HEX_SPRITES[i as usize], sprite);
        }
    }

    #[test]
    #[should_panic(expected = "Cannot provide address of sprite with value 0x10")]
    fn sprite_addressing_invalid() {
        let memory = RAM::new();

        memory.get_address_of_sprite(0x10);
    }
}
