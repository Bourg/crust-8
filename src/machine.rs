use crate::instruction::Instruction;
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

    pub fn step(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::StoreXNN { register, value } => {
                self.registers.set_register(*register, *value);
            }
            Instruction::AddXNN { register, value } => {
                let (value, carry) = self.registers.v[*register as usize].overflowing_add(*value);

                self.registers.set_register(*register, value);
                // TODO does this modify the carry, or only register-to-register addition?
                self.registers.set_flag(if carry { 1 } else { 0 })

                // TODO increment the program count
            }
            _ => panic!("Unsupported"),
        }
    }

    pub fn step_many<'a, T>(&mut self, instructions: T)
    where
        T: IntoIterator<Item = &'a Instruction>,
    {
        instructions
            .into_iter()
            .for_each(|instruction| self.step(instruction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction::*;

    #[test]
    fn store_xnn() {
        let mut machine = Machine::new();

        machine.step_many(&[
            StoreXNN {
                register: 0x7,
                value: 77,
            },
            StoreXNN {
                register: 0x5,
                value: 22,
            },
            StoreXNN {
                register: 0xF,
                value: 123,
            },
            StoreXNN {
                register: 0x5,
                value: 23,
            },
        ]);

        assert_eq!(
            [0, 0, 0, 0, 0, 23, 0, 77, 0, 0, 0, 0, 0, 0, 0, 123],
            machine.registers.v
        );
    }

    #[test]
    fn add_xnn() {
        let mut machine = Machine::new();

        let register = 0x5;

        assert_eq!(0, machine.registers.get_register(register));

        // A first add should work and not overflow
        machine.step(&AddXNN {
            register,
            value: 106,
        });
        assert_eq!(106, machine.registers.get_register(register));
        assert_eq!(0, machine.registers.get_flag());

        // A second add just below the limit should also not overflow
        machine.step(&Instruction::AddXNN {
            register,
            value: 149,
        });
        assert_eq!(255, machine.registers.get_register(register));
        assert_eq!(0, machine.registers.get_flag());

        // Add one more and it should overflow
        machine.step(&AddXNN { register, value: 1 });
        assert_eq!(0, machine.registers.get_register(register));
        assert_eq!(1, machine.registers.get_flag());

        // Add one last time and the flag should reset
        machine.step(&AddXNN { register, value: 3 });
        assert_eq!(3, machine.registers.get_register(register));
        assert_eq!(0, machine.registers.get_flag());
    }
}
