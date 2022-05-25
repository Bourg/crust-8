pub mod memory {
    use std::fmt::{Debug, Formatter};

    const ADDRESS_INTERPRETER_START: usize = 0x0;
    const ADDRESS_PROGRAM_START: usize = 0x200;
    const ADDRESS_MAX: usize = 0xFFF;

    type HexSprite = [u8; 5];

    const HEX_SPRITES: [HexSprite; 0x10] = [
        // Numerical 0-9
        [0xF0, 0x90, 0x90, 0x90, 0xF0],
        [0x20, 0x60, 0x20, 0x20, 0x70],
        [0xF0, 0x10, 0xF0, 0x80, 0xF0],
        [0xF0, 0x10, 0xF0, 0x10, 0xF0],
        [0x90, 0x90, 0xF0, 0x10, 0x10],
        [0xF0, 0x10, 0xF0, 0x10, 0xF0],
        [0xF0, 0x80, 0xF0, 0x90, 0xF0],
        [0xF0, 0x10, 0x20, 0x40, 0x40],
        [0xF0, 0x90, 0xF0, 0x90, 0xF0],
        [0xF0, 0x90, 0xF0, 0x10, 0xF0],

        // Alpha A-F
        [0xF0, 0x90, 0xF0, 0x90, 0x90],
        [0xE0, 0x90, 0xE0, 0x90, 0xE0],
        [0xF0, 0x80, 0x80, 0x80, 0x80],
        [0xE0, 0x90, 0x90, 0x90, 0x90],
        [0xF0, 0x80, 0xF0, 0x80, 0xF0],
        [0xF0, 0x80, 0xF0, 0x80, 0x80]
    ];

    pub struct RAM {
        value: [u8; ADDRESS_MAX],
    }

    impl RAM {
        pub fn new() -> RAM {
            let mut ram = RAM { value: [0; ADDRESS_MAX] };
            let mut write_start = ADDRESS_INTERPRETER_START;

            for sprite in HEX_SPRITES {
                let write_end = write_start + sprite.len();

                ram.value[write_start..write_end].copy_from_slice(&sprite);
                write_start = write_end;
            }

            ram
        }
    }

    impl Debug for RAM {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let bytes_per_line = 10;

            let mut bytes_written = 0;

            for byte in self.value {
                if bytes_written % bytes_per_line == 0 {
                    write!(f, "{:#05X}: ", bytes_written);
                }

                write!(f, "{:02X}", byte);
                bytes_written = bytes_written + 1;

                if bytes_written % bytes_per_line == 0 {
                    writeln!(f, "");
                } else {
                    write!(f, " ");
                }
            }

            Ok(())
        }
    }
}