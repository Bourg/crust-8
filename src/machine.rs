use crate::graphics;
use crate::instruction::Instruction;
use crate::register;
use crate::{memory, settings};

pub struct Machine<'a, G: graphics::Draw> {
    pub ram: memory::RAM,
    pub registers: register::Registers,
    pub graphics: G,
    pub settings: &'a settings::Settings,
}

// TODO where to put this
enum FlagSideEffect {
    NONE,
    SET(bool),
}

impl<'a, G> Machine<'a, G> where G: graphics::Draw {}

impl<'a> Machine<'a, graphics::HeadlessGraphics> {
    pub fn new() -> Machine<'a, graphics::HeadlessGraphics> {
        Machine::new_with_settings(&settings::Settings {
            bit_shift_mode: settings::BitShiftMode::OneRegister,
        })
    }

    pub fn new_with_settings(settings: &settings::Settings) -> Machine<graphics::HeadlessGraphics> {
        Machine {
            ram: memory::RAM::new(),
            registers: register::Registers::new(),
            graphics: graphics::HeadlessGraphics::new(),
            settings,
        }
    }

    pub fn step(&mut self, instruction: &Instruction) {
        let bit_shift_mode = self.settings.bit_shift_mode;

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
            Instruction::OrXY { target, source } => self.op(target, source, |tv, sv| tv | sv),
            Instruction::AndXY { target, source } => self.op(target, source, |tv, sv| tv & sv),
            Instruction::XorXY { target, source } => self.op(target, source, |tv, sv| tv ^ sv),
            Instruction::AddXY { target, source } => {
                self.flagging_op(target, source, |tv, sv| {
                    let (value, carry) = tv.overflowing_add(sv);
                    (value, FlagSideEffect::SET(carry))
                });
            }
            Instruction::SubXY { target, source } => {
                self.flagging_op(target, source, |tv, sv| {
                    let (value, borrow) = tv.overflowing_sub(sv);
                    (value, FlagSideEffect::SET(!borrow))
                });
            }
            Instruction::ShrXY { target, source } => self.flagging_op(target, source, |tv, sv| {
                let sv = if let settings::BitShiftMode::OneRegister = bit_shift_mode {
                    tv
                } else {
                    sv
                };
                (sv >> 1, FlagSideEffect::SET(sv % 2 == 1))
            }),
            Instruction::SUBXYReverse { target, source } => {
                self.flagging_op(target, source, |tv, sv| {
                    let (value, borrow) = sv.overflowing_sub(tv);
                    (value, FlagSideEffect::SET(!borrow))
                });
            }
            Instruction::ShlXY { target, source } => self.flagging_op(target, source, |tv, sv| {
                let sv = if let settings::BitShiftMode::OneRegister = bit_shift_mode {
                    tv
                } else {
                    sv
                };
                (sv << 1, FlagSideEffect::SET(sv & 0x80 != 0))
            }),
            Instruction::StoreNNN { value } => {
                self.registers.i = *value;
            }
        }
    }

    pub fn step_many<'b, T>(&mut self, instructions: T)
    where
        T: IntoIterator<Item = &'b Instruction>,
    {
        instructions
            .into_iter()
            .for_each(|instruction| self.step(instruction))
    }

    fn op<T>(&mut self, target: &u8, source: &u8, op: T)
    where
        T: Fn(u8, u8) -> u8,
    {
        self.flagging_op(target, source, |t, s| (op(t, s), FlagSideEffect::NONE))
    }

    fn flagging_op<T>(&mut self, target: &u8, source: &u8, op: T)
    where
        T: Fn(u8, u8) -> (u8, FlagSideEffect),
    {
        let source_value = self.registers.get_register(*source);
        let target_value = self.registers.get_register(*target);

        let (value, flag_effect) = op(target_value, source_value);

        self.registers.set_register(*target, value);

        if let FlagSideEffect::SET(flag) = flag_effect {
            self.registers.set_flag(if flag { 1 } else { 0 });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction::*;
    use crate::settings::BitShiftMode;

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

    #[test]
    fn sub() {
        let mut machine = Machine::new();

        machine.registers.set_register(0x0, 17);
        machine.registers.set_register(0x1, 30);
        machine.registers.set_flag(0xDD);

        // Subtraction without wrapping sets the flag to 1
        machine.step(&SubXY {
            target: 0x1,
            source: 0x0,
        });
        assert_eq!([17, 13], machine.registers.v[0..2]);
        assert_eq!(0x1, machine.registers.get_flag());

        // Subtraction with wrapping sets the flag to 0
        machine.step(&SubXY {
            target: 0x1,
            source: 0x0,
        });
        assert_eq!([17, 252], machine.registers.v[0..2]);
        assert_eq!(0x0, machine.registers.get_flag());
    }

    #[test]
    fn sub_reverse() {
        let mut machine = Machine::new();

        // No borrow
        machine.step_many([
            &StoreXNN {
                register: 0xA,
                value: 0x2D,
            },
            &StoreXNN {
                register: 0xB,
                value: 0x4B,
            },
            &SUBXYReverse {
                target: 0xA,
                source: 0xB,
            },
        ]);
        assert_eq!([0x1E, 0x4B], machine.registers.v[0xA..=0xB]);
        assert_eq!(0x1, machine.registers.get_flag());

        // With carry
        machine.step_many([
            &StoreXNN {
                register: 0xC,
                value: 0x4B,
            },
            &StoreXNN {
                register: 0xD,
                value: 0x2D,
            },
            &SUBXYReverse {
                target: 0xC,
                source: 0xD,
            },
        ]);

        assert_eq!([0xE2, 0x2D], machine.registers.v[0xC..=0xD]);
        assert_eq!(0x0, machine.registers.get_flag());
    }

    #[test]
    fn bitwise() {
        let mut machine = Machine::new();

        machine.registers.set_flag(0xBB);

        machine.registers.set_register(1, 0x2D);
        machine.registers.set_register(2, 0x4B);
        machine.step(&OrXY {
            target: 0x1,
            source: 0x2,
        });
        assert_eq!([0x6F, 0x4B], machine.registers.v[1..3]);

        machine.registers.set_register(3, 0x2D);
        machine.registers.set_register(4, 0x4B);
        machine.step(&AndXY {
            target: 0x3,
            source: 0x4,
        });
        assert_eq!([0x09, 0x4B], machine.registers.v[3..5]);

        machine.registers.set_register(5, 0x2D);
        machine.registers.set_register(6, 0x4B);
        machine.step(&XorXY {
            target: 0x5,
            source: 0x6,
        });
        assert_eq!([0x66, 0x4B], machine.registers.v[5..7]);

        assert_eq!(0xBB, machine.registers.get_flag());
    }

    #[test]
    fn bit_shifts() {
        let target = 0xE;
        let source = 0xD;

        let one_register_settings = &settings::Settings {
            bit_shift_mode: BitShiftMode::OneRegister,
        };
        let two_register_settings = &settings::Settings {
            bit_shift_mode: BitShiftMode::TwoRegister,
        };

        let shr = &ShrXY { target, source };
        let shl = &ShlXY { target, source };

        let cases = [
            (one_register_settings, shr, 0x2C, 0xFF, 0x16, 0x00),
            (one_register_settings, shr, 0x2D, 0xFF, 0x16, 0x01),
            (one_register_settings, shl, 0x2D, 0xFF, 0x5A, 0x00),
            (one_register_settings, shl, 0xAD, 0xFF, 0x5A, 0x01),
            (two_register_settings, shr, 0xFF, 0x2C, 0x16, 0x00),
            (two_register_settings, shr, 0xFF, 0x2D, 0x16, 0x01),
            (two_register_settings, shl, 0xFF, 0x2D, 0x5A, 0x00),
            (two_register_settings, shl, 0xFF, 0xAD, 0x5A, 0x01),
        ];

        for (settings, instruction, target_value, source_value, expected_output, expected_flag) in
            cases
        {
            let mut machine = Machine::new_with_settings(settings);

            machine.registers.set_flag(0xFF);
            machine.registers.set_register(target, target_value);
            machine.registers.set_register(source, source_value);
            machine.step(instruction);

            assert_eq!(expected_output, machine.registers.get_register(target));
            assert_eq!(source_value, machine.registers.get_register(source));
            assert_eq!(expected_flag, machine.registers.get_flag());
        }
    }

    #[test]
    fn store_nnn() {
        let mut machine = Machine::new();

        assert_eq!(0x0, machine.registers.i);

        machine.step(&StoreNNN { value: 0x4090 });

        assert_eq!([0u8; 16], machine.registers.v);
        assert_eq!(0x4090, machine.registers.i)
    }
}
