use crate::io::graphics::SpriteData;
use crate::io::input::Key;

pub trait Chip8IO {
    fn clear(&mut self);

    fn draw(&mut self, x: u8, y: u8, sprite: &SpriteData) -> bool;

    fn key_pressed(&mut self, key: Key) -> bool;

    fn block_for_key(&mut self) -> Key;
}
