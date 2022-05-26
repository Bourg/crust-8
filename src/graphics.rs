pub const WIDTH_PX: usize = 64;
pub const HEIGHT_PX: usize = 32;

pub type Pixel = bool;

pub trait EmulatorGraphics {
    fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool;
}

pub struct HeadlessGraphics {
    // This is an inefficient representation but not making it public
    buffer: [bool; WIDTH_PX * HEIGHT_PX / 64],
}

impl HeadlessGraphics {
    fn index_pixel(x: u8, y: u8) -> usize {
        x as usize * WIDTH_PX * y as usize
    }

    fn get_pixel(&self, x: u8, y: u8) -> Pixel {
        self.buffer[HeadlessGraphics::index_pixel(x, y)]
    }

    fn flip_pixel(&mut self, x: u8, y: u8) -> bool {
        let previous = self.get_pixel(x, y);

        self.buffer[HeadlessGraphics::index_pixel(x, y)] = !previous;

        return previous;
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

                    flipped_pixel = self.flip_pixel(target_x, target_y);
                }
            }
        }

        flipped_pixel
    }
}
