pub type Pixel = bool;
pub type SpriteData = [u8];

const WIDTH_PX: usize = 64;
const HEIGHT_PX: usize = 32;

pub struct GraphicsBuffer {
    buffer: [bool; WIDTH_PX * HEIGHT_PX],
}

impl GraphicsBuffer {
    pub fn new() -> GraphicsBuffer {
        GraphicsBuffer {
            buffer: [false; WIDTH_PX * HEIGHT_PX],
        }
    }

    pub fn width(&self) -> usize {
        WIDTH_PX
    }

    pub fn height(&self) -> usize {
        HEIGHT_PX
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = false;
        }
    }

    pub fn draw(&mut self, canvas_x: u8, canvas_y: u8, sprite: &[u8]) -> bool {
        let mut flipped_pixel = false;

        for (sprite_y, sprite_line) in sprite.iter().enumerate() {
            // Handle the bits backwards so the rightmost bit is the leftmost pixel
            let sprite_y = sprite_y as u8;
            let sprite_line = sprite_line.reverse_bits();

            // Each bit of the u8 is one column
            for sprite_x in 0..8 {
                // Draw the pixel if that bit of the sprite is on
                let pixel = (sprite_line & (1 << sprite_x)) != 0;

                if pixel {
                    let target_x = canvas_x.wrapping_add(sprite_x);
                    let target_y = canvas_y.wrapping_add(sprite_y);

                    flipped_pixel = self.flip_pixel(target_x, target_y) || flipped_pixel;
                }
            }
        }

        flipped_pixel
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> Option<Pixel> {
        GraphicsBuffer::index_pixel(x, y).map(|index| self.buffer[index])
    }

    fn flip_pixel(&mut self, x: u8, y: u8) -> bool {
        GraphicsBuffer::index_pixel(x, y)
            .map(|index| {
                let previous = self.buffer[index];
                self.buffer[index] = !previous;
                return previous;
            })
            .unwrap_or(false)
    }

    fn index_pixel(x: u8, y: u8) -> Option<usize> {
        let x = x as usize;
        let y = y as usize;

        if x >= WIDTH_PX || y >= HEIGHT_PX {
            None
        } else {
            Some(x + WIDTH_PX * y)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_pixel() {
        assert_eq!(None, GraphicsBuffer::index_pixel(32, 63));
        assert_eq!(None, GraphicsBuffer::index_pixel(31, 64));
        assert_eq!(None, GraphicsBuffer::index_pixel(32, 64));

        assert_eq!(0, GraphicsBuffer::index_pixel(0, 0).unwrap());
        assert_eq!(5, GraphicsBuffer::index_pixel(5, 0).unwrap());
        assert_eq!(63, GraphicsBuffer::index_pixel(63, 0).unwrap());
        assert_eq!(64, GraphicsBuffer::index_pixel(0, 1).unwrap());
        assert_eq!(140, GraphicsBuffer::index_pixel(12, 2).unwrap());
        assert_eq!(2047, GraphicsBuffer::index_pixel(63, 31).unwrap());
    }

    #[test]
    fn test_pixel_operations() {
        let mut graphics = GraphicsBuffer::new();

        // Should be off to start
        assert_eq!(Some(false), graphics.get_pixel(6, 1));
        graphics.buffer[70] = true;

        // Looking up the pixel by coordinate should be true now
        assert_eq!(Some(true), graphics.get_pixel(6, 1));

        // Flipping the pixel should return true because it was flipped off
        assert_eq!(true, graphics.flip_pixel(6, 1));

        // And the underlying pixel should now be false
        assert_eq!(Some(false), graphics.get_pixel(6, 1));

        // Flipping it back on should return false because nothing was turned off
        assert_eq!(false, graphics.flip_pixel(6, 1));
        assert_eq!(true, graphics.flip_pixel(6, 1));

        assert_eq!([false; 2048], graphics.buffer);
    }

    #[test]
    fn test_headless_graphics() {
        let mut graphics = GraphicsBuffer::new();

        // There should be 2048 boolean cells in the graphics buffer
        assert_eq!([false; 2048], graphics.buffer);

        // Drawing an empty sprite should not affect the buffer or indicate a flip
        let flipped = graphics.draw(0, 0, &[]);
        assert_eq!(false, flipped);
        assert_eq!([false; 2048], graphics.buffer);

        // This sprite forms a checkerboard pattern
        let sprite_positive = [0xAA, 0x55, 0xAA, 0x55];
        let sprite_negative = [0x55, 0xAA, 0x55, 0xAA];

        let pixels_aa = [true, false, true, false, true, false, true, false];
        let pixels_55 = [false, true, false, true, false, true, false, true];
        let pixels_00 = [false; 8];
        let pixels_ff = [true; 8];

        let flipped = graphics.draw(0, 0, &sprite_positive);
        assert_eq!(false, flipped);

        // Check the whole draw area
        assert_eq!(pixels_aa, graphics.buffer[0..8]);
        assert_eq!(pixels_55, graphics.buffer[64..72]);
        assert_eq!(pixels_aa, graphics.buffer[128..136]);
        assert_eq!(pixels_55, graphics.buffer[192..200]);

        // Check a few things outside the draw area
        assert_eq!(pixels_00, graphics.buffer[8..16]);
        assert_eq!(pixels_00, graphics.buffer[200..208]);

        // Draw the checkerboard's inverse
        let flipped = graphics.draw(0, 0, &sprite_negative);
        assert_eq!(false, flipped);
        for y in 0..4 {
            assert_eq!(pixels_ff, graphics.buffer[y * 64..y * 64 + 8])
        }

        // Flip both checkerboards off again, should reset the board
        assert_eq!(true, graphics.draw(0, 0, &sprite_positive));
        assert_eq!(true, graphics.draw(0, 0, &sprite_negative));
        assert_eq!([false; 2048], graphics.buffer);
    }

    #[test]
    fn headless_graphics_out_of_bounds() {
        // An 8x8 all-black sprite
        let sprite = [0xFF; 8];

        let mut graphics = GraphicsBuffer::new();

        let flipped = graphics.draw(WIDTH_PX as u8 - 1, HEIGHT_PX as u8 - 1, &sprite);
        assert_eq!(false, flipped);

        let mut expected = [false; 2048];
        expected[2047] = true;

        assert_eq!(expected, graphics.buffer);
    }
}
