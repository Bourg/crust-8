use std::error;
use std::fmt;
use std::sync::{Arc, Mutex};

pub const WIDTH_PX: usize = 64;
pub const HEIGHT_PX: usize = 32;

pub type Pixel = bool;

pub trait Draw {
    fn clear(&mut self);

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

    pub fn get_pixel(&self, x: u8, y: u8) -> Option<Pixel> {
        HeadlessGraphics::index_pixel(x, y).map(|index| self.buffer[index])
    }

    fn flip_pixel(&mut self, x: u8, y: u8) -> bool {
        HeadlessGraphics::index_pixel(x, y)
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

impl fmt::Display for HeadlessGraphics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..HEIGHT_PX {
            for x in 0..WIDTH_PX {
                let char = if self.get_pixel(x as u8, y as u8) == Some(true) {
                    '\u{25A3}'
                } else {
                    '\u{25A1}'
                };
                write!(f, "{}", char)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Draw for HeadlessGraphics {
    fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = false;
        }
    }

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

                    flipped_pixel = self.flip_pixel(target_x, target_y) || flipped_pixel;
                }
            }
        }

        flipped_pixel
    }
}

// TODO I'm really not super happy with the ownership structure for window graphics
#[derive(Clone)]
pub struct PistonGraphics {
    buffer: Arc<Mutex<HeadlessGraphics>>,
}

impl PistonGraphics {
    pub fn new() -> PistonGraphics {
        PistonGraphics {
            buffer: Arc::new(Mutex::new(HeadlessGraphics::new())),
        }
    }

    pub fn open_window(&self) -> Result<(), Box<dyn error::Error>> {
        let mut window: piston_window::PistonWindow =
            piston::WindowSettings::new("crust8", [640, 320]).build()?;

        while let Some(e) = window.next() {
            window.draw_2d(&e, move |c, g, _| {
                let buffer = self.buffer.lock().unwrap();

                graphics::clear([1.0, 1.0, 1.0, 1.0], g);

                for y in 0..HEIGHT_PX {
                    for x in 0..WIDTH_PX {
                        let pixel = buffer.get_pixel(x as u8, y as u8);

                        if pixel == Some(true) {
                            let start_x = 10.0 * x as f64;
                            let start_y = 10.0 * y as f64;

                            graphics::rectangle(
                                [0.0, 0.0, 0.0, 1.0],
                                [start_x, start_y, 10.0, 10.0],
                                c.transform,
                                g,
                            );
                        }
                    }
                }
            });
        }

        Ok(())
    }
}

impl Draw for PistonGraphics {
    // TODO do we need result types for these
    fn clear(&mut self) {
        self.buffer.lock().unwrap().clear();
    }

    fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        self.buffer.lock().unwrap().draw(x, y, sprite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_pixel() {
        assert_eq!(None, HeadlessGraphics::index_pixel(32, 63));
        assert_eq!(None, HeadlessGraphics::index_pixel(31, 64));
        assert_eq!(None, HeadlessGraphics::index_pixel(32, 64));

        assert_eq!(0, HeadlessGraphics::index_pixel(0, 0).unwrap());
        assert_eq!(5, HeadlessGraphics::index_pixel(5, 0).unwrap());
        assert_eq!(63, HeadlessGraphics::index_pixel(63, 0).unwrap());
        assert_eq!(64, HeadlessGraphics::index_pixel(0, 1).unwrap());
        assert_eq!(140, HeadlessGraphics::index_pixel(12, 2).unwrap());
        assert_eq!(2047, HeadlessGraphics::index_pixel(63, 31).unwrap());
    }

    #[test]
    fn test_pixel_operations() {
        let mut graphics = HeadlessGraphics::new();

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
        let mut graphics = HeadlessGraphics::new();

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

        let mut graphics = HeadlessGraphics::new();

        let flipped = graphics.draw(WIDTH_PX as u8 - 1, HEIGHT_PX as u8 - 1, &sprite);
        assert_eq!(false, flipped);

        let mut expected = [false; 2048];
        expected[2047] = true;

        assert_eq!(expected, graphics.buffer);
    }
}
