use crate::io::chip8_io::Chip8IO;
use crate::io::graphics::{GraphicsBuffer, SpriteData};
use crate::io::input::Key;

pub struct HeadlessIO {
    graphics_buffer: GraphicsBuffer,
}

impl HeadlessIO {
    pub fn new() -> Self {
        HeadlessIO {
            graphics_buffer: GraphicsBuffer::new(),
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

    fn key_pressed(&mut self, _key: Key) -> bool {
        // TODO can allow a preset key layout?
        panic!("Cannot read keys for headless inputs");
    }

    // TODO is there anything that can be done here?
    fn block_for_key(&mut self) -> Option<Key> {
        // TODO can allow a pre-programmed string of inputs?
        panic!("Cannot block for headless input");
    }
}
