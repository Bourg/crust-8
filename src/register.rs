use crate::memory;

const FLAG_REGISTER: u8 = 0xF;

pub struct Registers {
    pub v: [u8; 16],
    pub i: memory::Address,
    pub pc: memory::Address,
    pub sp: u8,
    // TODO delay and sound registers
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
        }
    }

    pub fn set_register(&mut self, index: u8, value: u8) {
        self.v[index as usize] = value;
    }

    pub fn get_register(&self, index: u8) -> u8 {
        self.v[index as usize]
    }

    pub fn set_flag(&mut self, value: u8) {
        self.set_register(FLAG_REGISTER, value);
    }

    pub fn get_flag(&self) -> u8 {
        self.get_register(FLAG_REGISTER)
    }
}
