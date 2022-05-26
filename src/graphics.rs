pub const WIDTH_PX: usize = 64;
pub const HEIGHT_PX: usize = 32;

pub type Pixel = bool;

pub trait EmulatorGraphics {
    fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool;
}

pub struct HeadlessGraphics {
    // This is an inefficient representation but not making it public
    buffer: [bool; WIDTH_PX * HEIGHT_PX],
}

impl HeadlessGraphics {
    pub fn new() -> HeadlessGraphics {
        HeadlessGraphics {
            buffer: [false; WIDTH_PX * HEIGHT_PX],
        }
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> Pixel {
        self.buffer[HeadlessGraphics::index_pixel(x, y)]
    }

    fn flip_pixel(&mut self, x: u8, y: u8) -> bool {
        let previous = self.get_pixel(x, y);

        self.buffer[HeadlessGraphics::index_pixel(x, y)] = !previous;

        return previous;
    }

    fn index_pixel(x: u8, y: u8) -> usize {
        x as usize + WIDTH_PX * y as usize
    }
}

impl EmulatorGraphics for HeadlessGraphics {
    fn draw(&mut self, canvas_x: u8, canvas_y: u8, sprite: &[u8]) -> bool {
        let mut flipped_pixel = false;

        for sprite_y in 0..sprite.len() as u8 {
            // Read the current line of the sprite
            let sprite_line = sprite[sprite_y as usize].reverse_bits();

            // Each bit of the u8 is one column
            for sprite_x in 0..8 {
                // Draw the pixel if that bit of the sprite is on
                let pixel = (sprite_line & (1 << sprite_x)) != 0;

                if pixel {
                    let target_x = canvas_x + sprite_x;
                    let target_y = canvas_y + sprite_y;

                    flipped_pixel = flipped_pixel || self.flip_pixel(target_x, target_y);
                }
            }
        }

        flipped_pixel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_pixel() {
        assert_eq!(0, HeadlessGraphics::index_pixel(0, 0));
        assert_eq!(5, HeadlessGraphics::index_pixel(5, 0));
        assert_eq!(63, HeadlessGraphics::index_pixel(63, 0));
        assert_eq!(64, HeadlessGraphics::index_pixel(0, 1));
        assert_eq!(140, HeadlessGraphics::index_pixel(12, 2));
        assert_eq!(2047, HeadlessGraphics::index_pixel(63, 31));
    }

    #[test]
    fn test_pixel_operations() {
        let mut graphics = HeadlessGraphics::new();

        // Should be off to start
        assert_eq!(false, graphics.get_pixel(6, 1));
        graphics.buffer[70] = true;

        // Looking up the pixel by coordinate should be true now
        assert_eq!(true, graphics.get_pixel(6, 1));

        // Flipping the pixel should return true because it was flipped off
        assert_eq!(true, graphics.flip_pixel(6, 1));

        // And the underlying pixel should now be false
        assert_eq!(false, graphics.get_pixel(6, 1));

        // Flipping it back on should return false because nothing was turned off
        assert_eq!(false, graphics.flip_pixel(6, 1));
        assert_eq!(true, graphics.flip_pixel(6, 1));

        assert_eq!([false; 2048], graphics.buffer);
    }

    #[test]
    fn test_headless_graphics() {
        let mut graphics = HeadlessGraphics::new();

        // There should be 2048 boolean cells in the graphics buffer
        assert_eq!([false; 2048], graphics.buffer)
    }
}
