pub const NUMBER_OF_KEYS: usize = 0x10;

/// Keys on a Chip8 Keyboard
/// Each value is the numeric value of the key as a hexadecimal digit
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    D0 = 0,
    D1 = 1,
    D2 = 2,
    D3 = 3,
    D4 = 4,
    D5 = 5,
    D6 = 6,
    D7 = 7,
    D8 = 8,
    D9 = 9,
    A = 0xA,
    B = 0xB,
    C = 0xC,
    D = 0xD,
    E = 0xE,
    F = 0xF,
}

/// Trait for things that can be mapped to a Chip8 key
pub trait MapKey {
    /// Map something into a Chip8 key
    /// Return a Some(key) if a mapping exists, or None if no mapping exists
    fn map_key(&self) -> Option<Key>;
}

impl MapKey for u8 {
    fn map_key(&self) -> Option<Key> {
        match self {
            0x0 => Some(Key::D0),
            0x1 => Some(Key::D1),
            0x2 => Some(Key::D2),
            0x3 => Some(Key::D3),
            0x4 => Some(Key::D4),
            0x5 => Some(Key::D5),
            0x6 => Some(Key::D6),
            0x7 => Some(Key::D7),
            0x8 => Some(Key::D8),
            0x9 => Some(Key::D9),
            0xA => Some(Key::A),
            0xB => Some(Key::B),
            0xC => Some(Key::C),
            0xD => Some(Key::D),
            0xE => Some(Key::E),
            0xF => Some(Key::F),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Key::*;
    use super::*;

    const ALL_KEYS: [Key; NUMBER_OF_KEYS] =
        [D0, D1, D2, D3, D4, D5, D6, D7, D8, D9, A, B, C, D, E, F];

    #[test]
    fn map_key_u8() {
        // A key's u8 value should map to its own key
        for key in ALL_KEYS {
            let intrinsic_value = key as u8;

            if let Some(mapped_key) = intrinsic_value.map_key() {
                let mapped_value = mapped_key as u8;
                assert_eq!(intrinsic_value, mapped_value);
                assert_eq!(key, mapped_key);
            } else {
                assert!(false);
            }
        }
    }
}
