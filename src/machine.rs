use crate::instruction::Instruction;
use crate::{graphics, random};
use crate::{memory, settings};
use crate::{register, timer};
use std::{error, thread};

// Chip8 runs instructions at 500Hz
// The timers decrement at 60Hz
// Therefore, we can decrement timers ~ every 8 instructions
// This isn't perfect, but can revisit later

pub struct Machine<G: graphics::Draw, R: random::RandomSource, T: timer::Timer> {
    ram: memory::RAM,
    registers: register::Registers,
    settings: settings::Settings,

    graphics: G,
    random: R,
    timer: T,
}

enum FlagSideEffect {
    NONE,
    SET(bool),
}

type RunResult = Result<(), Box<dyn error::Error>>;

impl<G, R, Tmr> Machine<G, R, Tmr>
where
    G: graphics::Draw,
    R: random::RandomSource,
    Tmr: timer::Timer,
{
    pub fn new(
        graphics: G,
        random: R,
        timer: Tmr,
        settings: settings::Settings,
    ) -> Machine<G, R, Tmr> {
        Machine {
            ram: memory::RAM::new(),
            registers: register::Registers::new(),
            settings,
            graphics,
            random,
            timer,
        }
    }

    pub fn load_program<T, U>(&mut self, loader: T) -> U
    where
        T: memory::ProgramLoader<Output = U>,
    {
        self.ram.load_program(loader)
    }

    pub fn run_program(&mut self) -> RunResult {
        loop {
            self.step_program()?;
        }
    }

    fn step_program(&mut self) -> RunResult {
        let instruction_bytes = self.ram.get_instruction(self.registers.pc);
        let instruction_u16 = ((instruction_bytes[0] as u16) << 8) + instruction_bytes[1] as u16;

        let instruction = Instruction::from_bytes(instruction_bytes);

        // TODO better place to apply clock speed?
        // TODO account for the actual amount of time the instruction took?
        if self.timer.should_tick() {
            self.registers.tick_timers();
        }

        if let settings::ClockSpeed::Limited { instruction_time } = self.settings.clock_speed {
            thread::sleep(instruction_time);
        }

        match (
            instruction.clone(),
            self.settings.on_unrecognized_instruction,
        ) {
            (Ok(instruction), _) => {
                self.step(&instruction);
                Ok(())
            }
            (Err(_), settings::OnUnrecognizedInstruction::Skip) => {
                // TODO remove printing
                println!(
                    "Unrecognized instruction {:#06X} at address {:#05X}",
                    instruction_u16, self.registers.pc
                );
                self.registers.advance_pc();
                Ok(())
            }
            (Err(_), settings::OnUnrecognizedInstruction::Halt) => {
                // TODO no idea why I have to do this but I can't return String as Error
                // But for some reason, using ? on the result works
                instruction?;
                Ok(())
            }
        }
    }

    fn step(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::ClearScreen => {
                self.graphics.clear();
                self.registers.advance_pc();
            }
            Instruction::JumpNNN { address } => {
                self.registers.pc = *address;
            }
            // TODO this is missing test coverage
            Instruction::SkipEqXNN { register, value } => {
                let register_value = self.registers.get_register(*register);

                self.registers.advance_pc();
                if register_value == *value {
                    self.registers.advance_pc();
                }
            }
            // TODO this is missing test coverage
            Instruction::SkipNeXNN { register, value } => {
                let register_value = self.registers.get_register(*register);

                self.registers.advance_pc();
                if register_value != *value {
                    self.registers.advance_pc();
                }
            }
            // TODO this is missing test coverage
            Instruction::SkipEqXY {
                register_x,
                register_y,
            } => {
                let value_x = self.registers.get_register(*register_x);
                let value_y = self.registers.get_register(*register_y);

                self.registers.advance_pc();
                if value_x == value_y {
                    self.registers.advance_pc();
                }
            }
            Instruction::StoreXNN { register, value } => {
                self.registers.set_register(*register, *value);
                self.registers.advance_pc();
            }
            Instruction::AddXNN { register, value } => {
                let (value, _) = self.registers.v[*register as usize].overflowing_add(*value);

                self.registers.set_register(*register, value);
                self.registers.advance_pc();
            }
            Instruction::StoreXY { target, source } => {
                let source_value = self.registers.get_register(*source);
                self.registers.set_register(*target, source_value);
                self.registers.advance_pc();
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
            Instruction::ShrXY { target, source } => {
                let bit_shift_mode = self.settings.bit_shift_mode;

                self.flagging_op(target, source, |tv, sv| {
                    let sv = if let settings::BitShiftMode::OneRegister = bit_shift_mode {
                        tv
                    } else {
                        sv
                    };
                    (sv >> 1, FlagSideEffect::SET(sv % 2 == 1))
                })
            }
            Instruction::SUBXYReverse { target, source } => {
                self.flagging_op(target, source, |tv, sv| {
                    let (value, borrow) = sv.overflowing_sub(tv);
                    (value, FlagSideEffect::SET(!borrow))
                });
            }
            Instruction::ShlXY { target, source } => {
                let bit_shift_mode = self.settings.bit_shift_mode;

                self.flagging_op(target, source, |tv, sv| {
                    let sv = if let settings::BitShiftMode::OneRegister = bit_shift_mode {
                        tv
                    } else {
                        sv
                    };
                    (sv << 1, FlagSideEffect::SET(sv & 0x80 != 0))
                })
            }
            // TODO this is missing test coverage
            Instruction::SkipNeXY {
                register_x,
                register_y,
            } => {
                let value_x = self.registers.get_register(*register_x);
                let value_y = self.registers.get_register(*register_y);

                self.registers.advance_pc();
                if value_x != value_y {
                    self.registers.advance_pc();
                }
            }
            Instruction::StoreNNN { value } => {
                self.registers.i = *value;
                self.registers.advance_pc();
            }
            Instruction::JumpV0 { address } => {
                let offset = self.registers.get_register(0);
                self.registers.pc = *address + offset as u16;
            }
            Instruction::Rand { register, mask } => {
                let random_number = self.random.gen();
                let random_number = random_number & mask;

                self.registers.set_register(*register, random_number);

                self.registers.advance_pc();
            }
            Instruction::DrawXYN {
                x_register,
                y_register,
                bytes,
            } => {
                let x = self.registers.get_register(*x_register);
                let y = self.registers.get_register(*y_register);

                let sprite_address = self.registers.i;
                let sprite = self.ram.get_sprite_at_address(sprite_address, *bytes);

                let flipped = self.graphics.draw(x, y, sprite);

                self.registers.set_flag(if flipped { 1 } else { 0 });
                self.registers.advance_pc();
            }
            // TODO untested
            Instruction::StoreDelayInX { register } => {
                self.registers
                    .set_register(*register, self.registers.get_register(*register));

                self.registers.advance_pc();
            }
            // TODO untested
            Instruction::SetDelayToX { register } => {
                let value = self.registers.get_register(*register);

                self.registers.dt = value;

                self.registers.advance_pc();
            }
            Instruction::AddIX { register } => {
                self.registers.i += self.registers.get_register(*register) as u16;
                self.registers.advance_pc();
            }
            Instruction::StoreSpriteX { register } => {
                let value = self.registers.get_register(*register);
                let address = self.ram.get_address_of_sprite(value);
                self.registers.i = address;

                self.registers.advance_pc();
            }
            Instruction::StoreDecimal { register } => {
                let value = self.registers.get_register(*register);
                let address = self.registers.i;

                let (high, mid, low) = to_decimal_digits(value);
                let memory = self.ram.address_mut(address);

                memory[0] = high;
                memory[1] = mid;
                memory[2] = low;

                self.registers.advance_pc();
            }
            Instruction::WriteToMemory { max_register } => {
                let address = &mut self.registers.i;
                let target_memory = &mut self.ram.address_mut(*address)[0..=*max_register as usize];
                let source_registers = &self.registers.v[0..=*max_register as usize];

                target_memory.copy_from_slice(source_registers);

                if let settings::MemoryMode::Advance = self.settings.memory_mode {
                    *address += *max_register as u16 + 1;
                }

                self.registers.advance_pc();
            }
            Instruction::ReadFromMemory { max_register } => {
                let address = &mut self.registers.i;
                let source_memory = &self.ram.address(*address)[0..=*max_register as usize];
                let target_registers = &mut self.registers.v[0..=*max_register as usize];

                target_registers.copy_from_slice(source_memory);

                if let settings::MemoryMode::Advance = self.settings.memory_mode {
                    *address += *max_register as u16 + 1;
                }

                self.registers.advance_pc();
            }
        }
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

        self.registers.advance_pc();
    }
}

fn to_decimal_digits(value: u8) -> (u8, u8, u8) {
    let high = (value / 100) % 10;
    let mid = (value / 10) % 10;
    let low = value % 10;

    (high, mid, low)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction::*;

    // Convenience constructors for test machines
    impl Machine<graphics::HeadlessGraphics, random::FixedRandomSource, timer::InstructionTimer> {
        pub fn new_headless(
        ) -> Machine<graphics::HeadlessGraphics, random::FixedRandomSource, timer::InstructionTimer>
        {
            Machine::new_headless_with_settings(
                random::FixedRandomSource::new(vec![0]),
                settings::Settings::default(),
            )
        }

        pub fn new_headless_with_settings(
            random: random::FixedRandomSource,
            settings: settings::Settings,
        ) -> Machine<graphics::HeadlessGraphics, random::FixedRandomSource, timer::InstructionTimer>
        {
            Machine {
                ram: memory::RAM::new(),
                registers: register::Registers::new(),
                graphics: graphics::HeadlessGraphics::new(),
                timer: timer::InstructionTimer::new(),
                random,
                settings,
            }
        }

        pub fn test_program_with_gas(
            &mut self,
            gas: usize,
            program: &Vec<Instruction>,
        ) -> RunResult {
            self.load_program(program);

            for _ in 0..gas {
                self.step_program()?;
            }

            Ok(())
        }

        pub fn test_program_linear(&mut self, program: &Vec<Instruction>) -> RunResult {
            self.test_program_with_gas(program.len(), program)
        }
    }

    #[test]
    fn run() {
        let mut machine = Machine::new_headless();

        // Load the machine with an empty program and run - should stop immediately
        machine.load_program(&[] as &[u8]);
        let result = machine.run_program();
        assert!(result.is_err());
        assert_eq!(0x200, machine.registers.pc);

        // Load a simple program that does some math
        machine.load_program(&vec![
            StoreXNN {
                register: 0,
                value: 24,
            },
            StoreXNN {
                register: 1,
                value: 26,
            },
            StoreXY {
                target: 2,
                source: 1,
            },
            AddXY {
                target: 2,
                source: 0,
            },
            AddXNN {
                register: 2,
                value: 10,
            },
            StoreXY {
                target: 3,
                source: 2,
            },
            ShlXY {
                target: 3,
                source: 3,
            },
            ShlXY {
                target: 3,
                source: 3,
            },
            SubXY {
                target: 3,
                source: 0,
            },
        ]);

        assert!(machine.run_program().is_err());
        assert_eq!(0x212, machine.registers.pc);
        assert_eq!([24, 26, 60, 216], machine.registers.v[0..4]);
    }

    #[test]
    fn store_xnn() {
        let mut machine = Machine::new_headless();

        machine
            .test_program_linear(&vec![
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
            ])
            .unwrap();

        assert_eq!(
            [0, 0, 0, 0, 0, 23, 0, 77, 0, 0, 0, 0, 0, 0, 0, 123],
            machine.registers.v
        );
    }

    #[test]
    fn store_xy() {
        let mut machine = Machine::new_headless();

        machine
            .test_program_linear(&vec![
                StoreXNN {
                    register: 0x5,
                    value: 90,
                },
                StoreXY {
                    target: 0xE,
                    source: 0x5,
                },
            ])
            .unwrap();

        assert_eq!(
            [0, 0, 0, 0, 0, 90, 0, 0, 0, 0, 0, 0, 0, 0, 90, 0],
            machine.registers.v
        );
    }

    #[test]
    fn add_xnn() {
        let mut machine = Machine::new_headless();

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
        let mut machine = Machine::new_headless();

        // Set a flag that should be clobbered by non-overflowing flag set
        machine.registers.set_flag(0xEE);

        // Add two numbers
        machine
            .test_program_linear(&vec![
                StoreXNN {
                    register: 0x0,
                    value: 23,
                },
                StoreXNN {
                    register: 0x1,
                    value: 45,
                },
                AddXY {
                    target: 0x0,
                    source: 0x1,
                },
            ])
            .unwrap();

        // The registers should have been set and the flag set to 0
        assert_eq!([68, 45], machine.registers.v[0..2]);
        assert_eq!(0x0, machine.registers.get_flag());

        // Make a few more additions and overflow will occur
        machine.step(&AddXY {
            target: 0x0,
            source: 0x0,
        });
        machine.step(&AddXY {
            target: 0x0,
            source: 0x0,
        });

        assert_eq!([16, 45], machine.registers.v[0..2]);
        assert_eq!(0x1, machine.registers.get_flag());
    }

    #[test]
    fn sub() {
        let mut machine = Machine::new_headless();

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
        let mut machine = Machine::new_headless();

        // No borrow
        machine
            .test_program_linear(&vec![
                StoreXNN {
                    register: 0xA,
                    value: 0x2D,
                },
                StoreXNN {
                    register: 0xB,
                    value: 0x4B,
                },
                SUBXYReverse {
                    target: 0xA,
                    source: 0xB,
                },
            ])
            .unwrap();
        assert_eq!([0x1E, 0x4B], machine.registers.v[0xA..=0xB]);
        assert_eq!(0x1, machine.registers.get_flag());

        // With carry
        let mut machine = Machine::new_headless();
        machine
            .test_program_linear(&vec![
                StoreXNN {
                    register: 0xC,
                    value: 0x4B,
                },
                StoreXNN {
                    register: 0xD,
                    value: 0x2D,
                },
                SUBXYReverse {
                    target: 0xC,
                    source: 0xD,
                },
            ])
            .unwrap();

        assert_eq!([0xE2, 0x2D], machine.registers.v[0xC..=0xD]);
        assert_eq!(0x0, machine.registers.get_flag());
    }

    #[test]
    fn bitwise() {
        let mut machine = Machine::new_headless();

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

        let one_register_settings =
            settings::Settings::default().with_bit_shift_mode(settings::BitShiftMode::OneRegister);
        let two_register_settings =
            settings::Settings::default().with_bit_shift_mode(settings::BitShiftMode::TwoRegister);

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
            let mut machine = Machine::new_headless_with_settings(
                random::FixedRandomSource::new(vec![0]),
                settings.clone(),
            );

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
        let mut machine = Machine::new_headless();

        assert_eq!(0x0, machine.registers.i);

        machine.step(&StoreNNN { value: 0x409 });

        assert_eq!([0u8; 16], machine.registers.v);
        assert_eq!(0x409, machine.registers.i)
    }

    #[test]
    fn add_ix() {
        let mut machine = Machine::new_headless();

        machine
            .test_program_linear(&vec![
                StoreNNN { value: 0xCDE },
                StoreXNN {
                    register: 7,
                    value: 0x11,
                },
                AddIX { register: 7 },
            ])
            .unwrap();

        assert_eq!(0xCEF, machine.registers.i);
    }

    #[test]
    fn store_sprite_x() {
        let mut machine = Machine::new_headless();

        machine
            .test_program_linear(&vec![
                StoreXNN {
                    register: 0xE,
                    value: 0xA,
                },
                StoreSpriteX { register: 0xE },
            ])
            .unwrap();

        assert_eq!(5 * 0xA, machine.registers.i);
    }

    #[test]
    fn store_decimal() {
        let mut machine = Machine::new_headless();
        let address = 0x222;

        machine
            .test_program_linear(&vec![
                StoreNNN { value: address },
                StoreXNN {
                    register: 8,
                    value: 159,
                },
                StoreDecimal { register: 8 },
            ])
            .unwrap();

        let loaded_memory = &machine.ram.address(address)[0..3];
        assert_eq!([1, 5, 9], loaded_memory);
    }

    #[test]
    fn rand() {
        let random_numbers = vec![0x1, 0xFF, 0xFF, 0xFF, 0b10101010];

        let mut machine = Machine::new_headless_with_settings(
            random::FixedRandomSource::new(random_numbers.clone()),
            settings::Settings::default(),
        );

        machine
            .test_program_linear(&vec![
                // 0x1 & 0xFF
                Rand {
                    register: 5,
                    mask: 0xFF,
                },
                // 0xFF & 0x00
                Rand {
                    register: 6,
                    mask: 0x00,
                },
                // 0xFF & 0xFF
                Rand {
                    register: 7,
                    mask: 0xFF,
                },
                // 0xFF & 0x5A
                Rand {
                    register: 8,
                    mask: 0x5A,
                },
                // 0b10101010 & 0b11001100
                Rand {
                    register: 9,
                    mask: 0b11001100,
                },
            ])
            .unwrap();

        assert_eq!(0x1, machine.registers.v[5]);
        assert_eq!(0x0, machine.registers.v[6]);
        assert_eq!(0xFF, machine.registers.v[7]);
        assert_eq!(0x5A, machine.registers.v[8]);
        assert_eq!(0b10001000, machine.registers.v[9]);
    }

    #[test]
    fn test_read_from_memory() {
        let start_memory = 0xEE0;

        // TODO need tests for NoAdvance mode
        let mut machine = Machine::new_headless();
        machine.settings = machine
            .settings
            .with_memory_mode(settings::MemoryMode::Advance);

        let memory = machine.ram.address_mut(start_memory);
        memory[0..5].copy_from_slice(&[1, 3, 5, 7, 9]);

        let program = vec![
            StoreNNN {
                value: start_memory,
            },
            StoreXNN {
                register: 5,
                value: 0x23,
            },
            ReadFromMemory { max_register: 5 },
        ];

        machine.test_program_linear(&program).unwrap();

        assert_eq!(0xEE6, machine.registers.i);
        assert_eq!([1, 3, 5, 7, 9, 0], machine.registers.v[0..=5]);
        assert_eq!([1, 3, 5, 7, 9], machine.ram.address(start_memory)[0..5]);
    }

    #[test]
    fn test_write_to_memory() {
        let start_memory = 0xABC;

        let program = vec![
            // Store a bunch of values in registers
            StoreXNN {
                register: 0,
                value: 0xA,
            },
            StoreXNN {
                register: 1,
                value: 0xB,
            },
            StoreXNN {
                register: 2,
                value: 0xC,
            },
            StoreXNN {
                register: 3,
                value: 0xD,
            },
            StoreXNN {
                register: 4,
                value: 0xE,
            },
            // Persist to memory
            StoreNNN {
                value: start_memory,
            },
            WriteToMemory { max_register: 4 },
        ];

        let mut machine = Machine::new_headless();
        machine.settings = machine
            .settings
            .with_memory_mode(settings::MemoryMode::Advance);

        machine.test_program_linear(&program).unwrap();

        assert_eq!(0xAC1, machine.registers.i);
        assert_eq!(
            [0xA, 0xB, 0xC, 0xD, 0xE],
            machine.ram.address(start_memory)[0..5]
        );
    }

    #[test]
    fn jump() {
        let mut machine = Machine::new_headless();

        machine
            .test_program_with_gas(
                3,
                &vec![
                    JumpNNN { address: 0x206 },
                    StoreXNN {
                        register: 1,
                        value: 0x1,
                    },
                    StoreXNN {
                        register: 2,
                        value: 0x2,
                    },
                    StoreXNN {
                        register: 1,
                        value: 0x3,
                    },
                    StoreXNN {
                        register: 2,
                        value: 0x4,
                    },
                ],
            )
            .unwrap();

        assert_eq!([0x3, 0x4], machine.registers.v[1..=2]);
        assert_eq!(0x20A, machine.registers.pc);
    }

    #[test]
    fn jump_v0() {
        let mut machine = Machine::new_headless();

        machine
            .test_program_with_gas(
                4,
                &vec![
                    StoreXNN {
                        register: 0,
                        value: 0x6,
                    },
                    JumpV0 { address: 0x202 },
                    StoreXNN {
                        register: 1,
                        value: 0x1,
                    },
                    StoreXNN {
                        register: 2,
                        value: 0x2,
                    },
                    StoreXNN {
                        register: 1,
                        value: 0x3,
                    },
                    StoreXNN {
                        register: 2,
                        value: 0x4,
                    },
                ],
            )
            .unwrap();

        assert_eq!([0x3, 0x4], machine.registers.v[1..=2]);
        assert_eq!(0x20C, machine.registers.pc);
    }

    #[test]
    fn test_to_decimal_digits() {
        assert_eq!((0, 0, 0), to_decimal_digits(0));
        assert_eq!((0, 0, 1), to_decimal_digits(1));
        assert_eq!((0, 1, 0), to_decimal_digits(10));
        assert_eq!((0, 7, 0), to_decimal_digits(70));
        assert_eq!((1, 0, 0), to_decimal_digits(100));
        assert_eq!((1, 2, 3), to_decimal_digits(123));
        assert_eq!((2, 5, 5), to_decimal_digits(255));
    }

    // TODO graphics integration test
    // TODO split the pub tests into integration test files
}
