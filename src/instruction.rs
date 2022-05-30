use crate::memory;
use Instruction::*;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    // 00E0
    ClearScreen,
    // TODO 00EE - RET (Category: Subroutines)
    // TODO 0nnn - SYS addr (Category: Subroutines)
    // 1NNN,
    Jump {
        address: memory::Address,
    },
    // TODO 2nnn - CALL addr (Category: Subroutines)
    // TODO 3xkk - SE Vx, byte (Flow control, skip if VX == NN)
    // TODO 4xkk - SNE Vx, byte (Flow control, skip if VX != NN)
    // TODO 5xy0 - SE Vx, Vy (Flow control, skip if VX == VY)
    // 6XNN
    StoreXNN {
        register: u8,
        value: u8,
    },
    // 7XNN
    AddXNN {
        register: u8,
        value: u8,
    },
    // 8XY0
    StoreXY {
        target: u8,
        source: u8,
    },
    // 8XY1
    OrXY {
        target: u8,
        source: u8,
    },
    // 8XY2
    AndXY {
        target: u8,
        source: u8,
    },
    // 8XY3
    XorXY {
        target: u8,
        source: u8,
    },
    // 8XY4
    AddXY {
        target: u8,
        source: u8,
    },
    // 8XY5
    SubXY {
        target: u8,
        source: u8,
    },
    // 8XY6
    ShrXY {
        target: u8,
        source: u8,
    },
    // 8XY7
    SUBXYReverse {
        target: u8,
        source: u8,
    },
    // 8XYE
    ShlXY {
        target: u8,
        source: u8,
    },
    // TODO 9xy0 - SNE Vx, Vy (Flow control, skip if VX != VY)
    // ANNN
    StoreNNN {
        value: memory::Address,
    },
    // Bnnn
    JumpV0 {
        address: memory::Address,
    },
    // CXNN
    Rand {
        register: u8,
        mask: u8,
    },
    // DXYN
    DrawXYN {
        x_register: u8,
        y_register: u8,
        bytes: u8,
    },
    // TODO Ex9E - SKP Vx (Input / flow control, skip if key in VX is pressed)
    // TODO ExA1 - SKNP Vx (Input / flow control, Skip if key in VX is not pressed)

    // TODO Fx07 - LD Vx, DT (Delay timer, store delay timer in VX)
    // TODO Fx0A - LD Vx, K (Input, wait for keypress, store in VX)
    // TODO Fx15 - LD DT, Vx (Delay timer, set delay timer to value of VX)
    // TODO Fx18 - LD ST, Vx (Sound, set sound timer to value of VX)
    // FX1E
    AddIX {
        register: u8,
    },
    // FX29
    StoreSpriteX {
        register: u8,
    },
    // FX33
    StoreDecimal {
        register: u8,
    },
    // FX55
    WriteToMemory {
        max_register: u8,
    },
    // FX65
    ReadFromMemory {
        max_register: u8,
    },
}

type InstructionBytes = [u8; 2];

impl Instruction {
    pub fn from_bytes(bytes: &[u8]) -> Result<Instruction, String> {
        if bytes.len() != 2 {
            return Err(String::from("Instructions must be exactly 2 bytes"));
        }

        let left = bytes[0];
        let right = bytes[1];

        match left >> 4 & 0xF {
            0 => {
                if left == 0 && right == 0xE0 {
                    Ok(ClearScreen)
                } else {
                    err_unsupported_instruction(left, right)
                }
            }
            1 => Ok(Jump {
                address: (((left & 0xF) as u16) << 8) + right as u16,
            }),
            6 => {
                let register = left & 0xF;
                let value = right;

                Ok(StoreXNN { register, value })
            }
            7 => {
                let register = left & 0xF;
                let value = right;

                Ok(AddXNN { register, value })
            }
            8 => {
                let target = left & 0xF;
                let source = right >> 4;

                match right & 0xF {
                    0 => Ok(StoreXY { target, source }),
                    1 => Ok(OrXY { target, source }),
                    2 => Ok(AndXY { target, source }),
                    3 => Ok(XorXY { target, source }),
                    4 => Ok(AddXY { target, source }),
                    5 => Ok(SubXY { target, source }),
                    6 => Ok(ShrXY { target, source }),
                    7 => Ok(SUBXYReverse { target, source }),
                    0xE => Ok(ShlXY { target, source }),
                    _ => err_unsupported_instruction(left, right),
                }
            }
            0xA => {
                let value = ((left as u16) << 8) & 0x0F00;
                let value = value + right as u16;

                Ok(StoreNNN { value })
            }
            0xB => Ok(JumpV0 {
                address: (((left & 0xF) as u16) << 8) + right as u16,
            }),
            0xC => Ok(Rand {
                register: left & 0xF,
                mask: right,
            }),
            0xD => Ok(DrawXYN {
                x_register: left & 0xF,
                y_register: right >> 4,
                bytes: right & 0xF,
            }),
            0xF => {
                let register = left & 0xF;

                match right {
                    0x1E => Ok(AddIX { register }),
                    0x29 => Ok(StoreSpriteX { register }),
                    0x33 => Ok(StoreDecimal { register }),
                    0x55 => Ok(WriteToMemory {
                        max_register: left & 0xF,
                    }),
                    0x65 => Ok(ReadFromMemory {
                        max_register: left & 0xF,
                    }),
                    _ => err_unsupported_instruction(left, right),
                }
            }
            _ => Err(format!(
                "Unsupported instruction {:#06X}",
                ((left as u16) << 8) + right as u16
            )),
        }
    }

    pub fn to_bytes(&self) -> InstructionBytes {
        match self {
            ClearScreen => [0x00, 0xE0],
            Jump { address } => from_u12(0x1, *address),
            StoreXNN {
                register,
                value: amount,
            } => [u4_to_u8(6, *register), *amount],
            AddXNN {
                register,
                value: amount,
            } => [u4_to_u8(7, *register), *amount],
            StoreXY {
                target: target_register,
                source: source_register,
            } => [u4_to_u8(8, *target_register), u4_to_u8(*source_register, 0)],
            OrXY { target, source } => from_u4s(8, *target, *source, 1),
            AndXY { target, source } => from_u4s(8, *target, *source, 2),
            XorXY { target, source } => from_u4s(8, *target, *source, 3),
            AddXY { target, source } => from_u4s(8, *target, *source, 4),
            SubXY { target, source } => from_u4s(8, *target, *source, 5),
            ShrXY { target, source } => from_u4s(8, *target, *source, 6),
            SUBXYReverse { target, source } => from_u4s(8, *target, *source, 7),
            ShlXY { target, source } => from_u4s(8, *target, *source, 0xE),
            StoreNNN { value } => from_u12(0xA, *value),
            JumpV0 { address } => from_u12(0xB, *address),
            Rand { register, mask } => [u4_to_u8(0xC, *register), *mask],
            DrawXYN {
                x_register,
                y_register,
                bytes,
            } => from_u4s(0xD, *x_register, *y_register, *bytes),
            AddIX { register } => from_u4s(0xF, *register, 0x1, 0xE),
            StoreSpriteX { register } => from_u4s(0xF, *register, 0x2, 0x9),
            StoreDecimal { register } => from_u4s(0xF, *register, 0x3, 0x3),
            // TODO find the canonical names for these instructions, names are getting twisted
            WriteToMemory { max_register } => from_u4s(0xF, *max_register, 0x5, 0x5),
            ReadFromMemory { max_register } => from_u4s(0xF, *max_register, 0x6, 0x5),
        }
    }

    pub fn to_u16(&self) -> u16 {
        let bytes = self.to_bytes();

        ((bytes[0] as u16) << 8) + bytes[1] as u16
    }
}

impl memory::ProgramLoader for &Vec<Instruction> {
    fn load_into_ram(&self, ram: &mut [u8]) {
        let bytes: Vec<u8> = self
            .iter()
            .flat_map(|instruction| instruction.to_bytes().into_iter())
            .collect();

        let bytes_slice: &[u8] = &bytes[..];

        bytes_slice.load_into_ram(ram);
    }
}

fn err_unsupported_instruction<T>(left: u8, right: u8) -> Result<T, String> {
    Err(format!(
        "Unsupported instruction {:#06X}",
        ((left as u16) << 8) + right as u16
    ))
}

fn from_u4s(a: u8, b: u8, c: u8, d: u8) -> InstructionBytes {
    [u4_to_u8(a, b), u4_to_u8(c, d)]
}

fn from_u12(leading_nibble: u8, value: u16) -> InstructionBytes {
    let prefix = (leading_nibble as u16) << 12;
    let value = prefix + (value & 0xFFF);

    [(value >> 8) as u8, (value & 0xFF) as u8]
}

fn u4_to_u8(most_significant: u8, least_significant: u8) -> u8 {
    let (most_significant, _) = most_significant.overflowing_mul(0x10);
    let least_significant = least_significant % 0x10;

    most_significant + least_significant
}

#[cfg(test)]
mod tests {
    use super::*;

    static CASES: &[(u16, Instruction)] = &[
        (0x00E0, ClearScreen),
        (0x1CDC, Jump { address: 0xCDC }),
        (
            0x6ABC,
            StoreXNN {
                register: 0xA,
                value: 0xBC,
            },
        ),
        (
            0x7ABC,
            AddXNN {
                register: 0xA,
                value: 0xBC,
            },
        ),
        (
            0x8AB0,
            StoreXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB1,
            OrXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB2,
            AndXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB3,
            XorXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB4,
            AddXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB5,
            SubXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB6,
            ShrXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8AB7,
            SUBXYReverse {
                target: 0xA,
                source: 0xB,
            },
        ),
        (
            0x8ABE,
            ShlXY {
                target: 0xA,
                source: 0xB,
            },
        ),
        (0xA1F2, StoreNNN { value: 0x1F2 }),
        (0xBDCD, JumpV0 { address: 0xDCD }),
        (
            0xD789,
            DrawXYN {
                x_register: 7,
                y_register: 8,
                bytes: 9,
            },
        ),
        (
            0xC345,
            Rand {
                register: 0x3,
                mask: 0x45,
            },
        ),
        (0xFE1E, AddIX { register: 0xE }),
        (0xFA29, StoreSpriteX { register: 0xA }),
        (0xFB33, StoreDecimal { register: 0xB }),
        (0xF055, WriteToMemory { max_register: 0x0 }),
        (0xFE65, ReadFromMemory { max_register: 0xE }),
    ];

    #[test]
    fn test_from_bytes() {
        for (input, expected) in CASES {
            let input = [(*input >> 8) as u8, (*input & 0x00FF) as u8];

            let actual = Instruction::from_bytes(&input);

            assert_eq!(*expected, actual.unwrap());
        }
    }

    #[test]
    fn test_to_bytes() {
        for (expected, input) in CASES {
            let actual = input.to_u16();

            assert_eq!(*expected, actual);
        }
    }

    #[test]
    fn load_instructions_as_program() {
        let mut memory = memory::RAM::new();
        let program = vec![
            AddXNN {
                register: 8,
                value: 0x99,
            },
            StoreXY {
                target: 0xA,
                source: 8,
            },
        ];
        memory.load_program(&program);
        assert_eq!(
            [0x78, 0x99, 0x8A, 0x80, 0x00],
            memory.program_memory()[0..5]
        );
    }

    #[test]
    fn test_u4_to_u8() {
        assert_eq!(0, u4_to_u8(0, 0));
        assert_eq!(0x3C, u4_to_u8(0x3, 0xC));
        assert_eq!(0xBD, u4_to_u8(0xAB, 0xCD));
    }

    #[test]
    fn test_u4s() {
        assert_eq!([0, 0], from_u4s(0xF0, 0xF0, 0xF0, 0xF0));
        assert_eq!([0xAB, 0xCD], from_u4s(0xA, 0xB, 0xC, 0xD));
        assert_eq!([0xAB, 0xCD], from_u4s(0xFA, 0xFB, 0xFC, 0xFD));
    }

    #[test]
    fn test_from_u12() {
        assert_eq!([0, 0], from_u12(0, 0));
        assert_eq!([0xAB, 0xCD], from_u12(0xA, 0xFBCD));
    }

    // TODO test cases:
    // - What should happen with carrying operations that are themselves operating on the carry?
    //   For example, subtracting register 0x1 from 0xF when 0x1 is larger - what should 0xF be?
}
