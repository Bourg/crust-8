use Instruction::*;

pub enum Instruction {
    // 6XNN
    StoreXNN { register: u8, value: u8 },
    // 7XNN
    AddXNN { register: u8, value: u8 },
    // 8XY0
    StoreXY { target: u8, source: u8 },
    // 8XY4
    AddXY { target: u8, source: u8 },
    // 8XY5
    SubXY { target: u8, source: u8 },
    // 8XY7
    // TODO better naming
    SUBXYReverse { target: u8, source: u8 },
}

impl Instruction {
    pub fn to_bytes(&self) -> [u8; 2] {
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
            _ => panic!("Not supported"), // Do we even need to_bytes?
        }
    }

    pub fn to_u16(&self) -> u16 {
        let bytes = self.to_bytes();

        ((bytes[0] as u16) << 8) + bytes[1] as u16
    }
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
        assert_eq!(
            0x6ABC,
            StoreXNN {
                register: 0xA,
                value: 0xBC,
            }
            .to_u16()
        );
        assert_eq!(
            0x8370,
            StoreXY {
                target: 3,
                source: 7,
            }
            .to_u16()
        )
    }

    #[test]
    fn test_u4_to_u8() {
        assert_eq!(0, u4_to_u8(0, 0));
        assert_eq!(0x3C, u4_to_u8(0x3, 0xC));
        assert_eq!(0xBD, u4_to_u8(0xAB, 0xCD));
    }
}
