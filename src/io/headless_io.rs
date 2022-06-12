use crate::io::chip8_io::Chip8IO;
use crate::io::graphics::{GraphicsBuffer, SpriteData};
use crate::io::input::{Key, Keypad};

pub struct HeadlessIO {
    pub graphics_buffer: GraphicsBuffer,
    pub keypad: Keypad,
    pub interrupt_key: Option<Key>,
}

impl HeadlessIO {
    pub fn new() -> Self {
        HeadlessIO {
            graphics_buffer: GraphicsBuffer::new(),
            keypad: Keypad::new(),
            interrupt_key: None,
        }
    }
}

impl Chip8IO for HeadlessIO {
    fn clear(&mut self) {
        self.graphics_buffer.clear();
    }

    fn draw(&mut self, x: u8, y: u8, sprite: &SpriteData) -> bool {
        self.graphics_buffer.draw(x, y, sprite)
    }

    fn key_pressed(&mut self, key: Key) -> bool {
        self.keypad.is_pressed(&key)
    }

    fn block_for_key(&mut self) -> Option<Key> {
        self.interrupt_key
    }
}
