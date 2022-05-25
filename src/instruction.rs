use Instruction::*;

pub enum Instruction {
    ADD { register: u8, amount: u8 },
}

impl Instruction {
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            // 7XNN
            ADD { register, amount } => [u4_to_u8(7, *register), *amount],
        }
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
    fn test_u4_to_u8() {
        assert_eq!(0, u4_to_u8(0, 0));
        assert_eq!(0x3C, u4_to_u8(0x3, 0xC));
        assert_eq!(0xBD, u4_to_u8(0xAB, 0xCD));
    }
}
