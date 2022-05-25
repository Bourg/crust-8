use crate::instruction;
use crate::memory;
use crate::register;

pub struct Machine {
    pub ram: memory::RAM,
    pub registers: register::Registers,
}

impl Machine {
    pub fn step(&mut self, instruction: instruction::Instruction) {
        match instruction {
            instruction::Instruction::ADD { register, amount } => {
                let (value, carry) = self.registers.v[register as usize].overflowing_add(amount);

                self.registers.set_register(register, value);
                self.registers.set_flag(if carry { 1 } else { 0 })
            }
        }
    }
}
