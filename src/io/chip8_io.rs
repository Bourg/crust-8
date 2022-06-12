use crate::io::graphics::SpriteData;
use crate::io::input::Key;
use std::error::Error;

// TODO can I refine this
pub type Chip8IOResult<T = ()> = Result<T, Box<dyn Error>>;

pub trait Chip8IO {
    fn clear(&mut self) -> Chip8IOResult;

    fn draw(&mut self, x: u8, y: u8, sprite: &SpriteData) -> Chip8IOResult<bool>;

    fn key_pressed(&mut self, key: Key) -> Chip8IOResult<bool>;

    fn block_for_key(&mut self) -> Chip8IOResult<Key>;
}
