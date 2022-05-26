use crate::memory;
use Instruction::*;

pub enum Instruction {
    // 6XNN
    StoreXNN { register: u8, value: u8 },
    // 7XNN
    AddXNN { register: u8, value: u8 },
    // 8XY0
    StoreXY { target: u8, source: u8 },
    // 8XY1
    OrXY { target: u8, source: u8 },
    // 8XY2
    AndXY { target: u8, source: u8 },
    // 8XY3
    XorXY { target: u8, source: u8 },
    // 8XY4
    AddXY { target: u8, source: u8 },
    // 8XY5
    SubXY { target: u8, source: u8 },
    // 8XY6
    ShrXY { target: u8, source: u8 },
    // 8XY7
    // TODO better naming
    SUBXYReverse { target: u8, source: u8 },
    // 8XYE
    ShlXY { target: u8, source: u8 },
    // ANNN
    StoreNNN { value: memory::Address },
}

type InstructionBytes = [u8; 2];

impl Instruction {
    pub fn to_bytes(&self) -> InstructionBytes {
        match self {
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
            StoreNNN { value } => from_u16(0x8000 + value & 0xFFF),
        }
    }

    pub fn to_u16(&self) -> u16 {
        let bytes = self.to_bytes();

        ((bytes[0] as u16) << 8) + bytes[1] as u16
    }
}

fn from_u4s(a: u8, b: u8, c: u8, d: u8) -> InstructionBytes {
    [u4_to_u8(a, b), u4_to_u8(c, d)]
}

fn from_u16(value: u16) -> InstructionBytes {
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

    #[test]
    fn test_to_bytes() {
        let cases = [
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
        ];

        for (expected, input) in cases {
            let actual = input.to_u16();

            assert_eq!(expected, actual);
        }
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
    fn test_from_u16() {
        assert_eq!([0, 0], from_u16(0));
        assert_eq!([0xAB, 0xCD], from_u16(0xABCD));
        assert_eq!([0, 0], from_u16(0));
    }

    // TODO test cases:
    // - What should happen with carrying operations that are themselves operating on the carry?
    //   For example, subtracting register 0x1 from 0xF when 0x1 is larger - what should 0xF be?
}
