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
                let (value, _) = self.registers.v[*register as usize].overflowing_add(*value);

                self.registers.set_register(*register, value);
            }
            Instruction::StoreXY { target, source } => {
                let source_value = self.registers.get_register(*source);
                self.registers.set_register(*target, source_value);
            }
            Instruction::AddXY { target, source } => {
                let source_value = self.registers.get_register(*source);
                let target_value = self.registers.get_register(*target);

                let (value, carry) = source_value.overflowing_add(target_value);

                self.registers.set_register(*target, value);
                self.registers.set_flag(if carry { 1 } else { 0 });
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
    fn store_xy() {
        let mut machine = Machine::new();

        machine.step_many([
            &StoreXNN {
                register: 0x5,
                value: 90,
            },
            &StoreXY {
                target: 0xE,
                source: 0x5,
            },
        ]);

        assert_eq!(
            [0, 0, 0, 0, 0, 90, 0, 0, 0, 0, 0, 0, 0, 0, 90, 0],
            machine.registers.v
        );
    }

    #[test]
    fn add_xnn() {
        let mut machine = Machine::new();

        // Numeric add should not affect the flag, so place a value there to ensure it isn't hurt
        let expected_flag = 0x8F;
        machine.registers.set_flag(expected_flag);

        let register = 0x5;

        assert_eq!(0, machine.registers.get_register(register));

        // A first add should work and not overflow
        machine.step(&AddXNN {
            register,
            value: 106,
        });
        assert_eq!(106, machine.registers.get_register(register));
        assert_eq!(expected_flag, machine.registers.get_flag());

        // A second add just below the limit should also not overflow
        machine.step(&Instruction::AddXNN {
            register,
            value: 149,
        });
        assert_eq!(255, machine.registers.get_register(register));
        assert_eq!(expected_flag, machine.registers.get_flag());

        // Add one more and it should overflow
        machine.step(&AddXNN { register, value: 1 });
        assert_eq!(0, machine.registers.get_register(register));
        assert_eq!(expected_flag, machine.registers.get_flag());

        // Add one last time and the flag should reset
        machine.step(&AddXNN { register, value: 3 });
        assert_eq!(3, machine.registers.get_register(register));
        assert_eq!(expected_flag, machine.registers.get_flag());
    }

    #[test]
    fn add_xy() {
        let mut machine = Machine::new();

        // Set a flag that should be clobbered by non-overflowing flag set
        machine.registers.set_flag(0xEE);

        // Add two numbers
        machine.step_many([
            &StoreXNN {
                register: 0x0,
                value: 23,
            },
            &StoreXNN {
                register: 0x1,
                value: 45,
            },
            &AddXY {
                target: 0x0,
                source: 0x1,
            },
        ]);

        // The registers should have been set and the flag set to 0
        assert_eq!([68, 45], machine.registers.v[0..2]);
        assert_eq!(0x0, machine.registers.get_flag());

        // Make a few more additions and overflow will occur
        machine.step_many([
            &AddXY {
                target: 0x0,
                source: 0x0,
            },
            &AddXY {
                target: 0x0,
                source: 0x0,
            },
        ]);

        assert_eq!([16, 45], machine.registers.v[0..2]);
        assert_eq!(0x1, machine.registers.get_flag());
    }
}
