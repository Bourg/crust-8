use crate::instruction;
use crate::memory;
use crate::register;

pub struct Machine {
    pub ram: memory::RAM,
    pub registers: register::Registers,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            ram: memory::RAM::new(),
            registers: register::Registers::new(),
        }
    }

    pub fn step(&mut self, instruction: instruction::Instruction) {
        match instruction {
            instruction::Instruction::ADD { register, amount } => {
                let (value, carry) = self.registers.v[register as usize].overflowing_add(amount);

                self.registers.set_register(register, value);
                self.registers.set_flag(if carry { 1 } else { 0 })

                // TODO increment the program count
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut machine = Machine::new();

        let register = 5u8;

        assert_eq!(0, machine.registers.get_register(register));

        // A first add should work and not overflow
        machine.step(instruction::Instruction::ADD {
            register,
            amount: 106,
        });
        assert_eq!(106, machine.registers.get_register(register));
        assert_eq!(0, machine.registers.get_flag());

        // A second add just below the limit should also not overflow
        machine.step(instruction::Instruction::ADD {
            register,
            amount: 149,
        });
        assert_eq!(255, machine.registers.get_register(register));
        assert_eq!(0, machine.registers.get_flag());

        // Add one more and it should overflow
        machine.step(instruction::Instruction::ADD {
            register,
            amount: 1,
        });
        assert_eq!(0, machine.registers.get_register(register));
        assert_eq!(1, machine.registers.get_flag());

        // Add one last time and the flag should reset
        machine.step(instruction::Instruction::ADD {
            register,
            amount: 3,
        });
        assert_eq!(3, machine.registers.get_register(register));
        assert_eq!(0, machine.registers.get_flag());
    }
}
