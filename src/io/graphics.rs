use crate::io::key::{Key, MapKey, NUMBER_OF_KEYS};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::RenderEvent;
use piston::RenderArgs;
use std::error;
use std::fmt;
use std::ops::DerefMut;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

// TODO decompose this

// TODO rename to io / split graphics and keys

pub const WIDTH_PX: usize = 64;
pub const HEIGHT_PX: usize = 32;

pub type Pixel = bool;

// TODO actually need to play audio when ST > 1

impl MapKey for piston::input::Button {
    fn map_key(&self) -> Option<Key> {
        match self {
            piston::Button::Keyboard(piston_key) => piston_key.map_key(),
            _ => None,
        }
    }
}

// TODO move
impl MapKey for piston::input::Key {
    fn map_key(&self) -> Option<Key> {
        match self {
            piston::input::Key::D1 => Some(Key::D1),
            piston::input::Key::D2 => Some(Key::D2),
            piston::input::Key::D3 => Some(Key::D3),
            piston::input::Key::D4 => Some(Key::C),
            piston::input::Key::Q => Some(Key::D4),
            piston::input::Key::W => Some(Key::D5),
            piston::input::Key::E => Some(Key::D6),
            piston::input::Key::R => Some(Key::D),
            piston::input::Key::A => Some(Key::D7),
            piston::input::Key::S => Some(Key::D8),
            piston::input::Key::D => Some(Key::D9),
            piston::input::Key::F => Some(Key::E),
            piston::input::Key::Z => Some(Key::A),
            piston::input::Key::X => Some(Key::D0),
            piston::input::Key::C => Some(Key::B),
            piston::input::Key::V => Some(Key::F),
            piston::input::Key::J => Some(Key::A),
            _ => None,
        }
    }
}

pub trait Chip8IO {
    fn clear(&mut self);

    fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool;

    fn read_key(&mut self, key: Key) -> bool;

    fn block_for_key(&mut self, sender: Sender<Key>);
}

pub struct HeadlessIO {
    key_buffer: [bool; NUMBER_OF_KEYS],
    video_buffer: [bool; WIDTH_PX * HEIGHT_PX],
}

impl HeadlessIO {
    pub fn new() -> HeadlessIO {
        HeadlessIO {
            key_buffer: [false; NUMBER_OF_KEYS],
            video_buffer: [false; WIDTH_PX * HEIGHT_PX],
        }
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> Option<Pixel> {
        HeadlessIO::index_pixel(x, y).map(|index| self.video_buffer[index])
    }

    fn flip_pixel(&mut self, x: u8, y: u8) -> bool {
        HeadlessIO::index_pixel(x, y)
            .map(|index| {
                let previous = self.video_buffer[index];
                self.video_buffer[index] = !previous;
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

impl fmt::Display for HeadlessIO {
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

impl Chip8IO for HeadlessIO {
    fn clear(&mut self) {
        for pixel in self.video_buffer.iter_mut() {
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
                    // TODO should add with wrap to avoid poisoning the lock
                    let target_x = canvas_x + sprite_x;
                    let target_y = canvas_y + sprite_y;

                    flipped_pixel = self.flip_pixel(target_x, target_y) || flipped_pixel;
                }
            }
        }

        flipped_pixel
    }

    fn read_key(&mut self, key: Key) -> bool {
        self.key_buffer.get(key as usize).cloned().unwrap_or(false)
    }

    // TODO is there anything that can be done here?
    fn block_for_key(&mut self, _sender: Sender<Key>) {
        panic!("Cannot block for headless input");
    }
}

// TODO I'm really not super happy with the ownership structure for window graphics
#[derive(Clone)]
pub struct PistonGraphics {
    headless: Arc<Mutex<HeadlessIO>>,
    // TODO this is gross
    key_interrupt: Arc<Mutex<Option<Sender<Key>>>>,
}

impl PistonGraphics {
    pub fn new() -> PistonGraphics {
        PistonGraphics {
            headless: Arc::new(Mutex::new(HeadlessIO::new())),
            key_interrupt: Arc::new(Mutex::new(None)),
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        gl.draw(args.viewport(), |c, gl| {
            let buffer = self.headless.lock().unwrap();

            graphics::clear([1.0, 1.0, 1.0, 1.0], gl);

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
                            gl,
                        );
                    }
                }
            }
        })
    }

    pub fn open_window(&self) -> Result<(), Box<dyn error::Error>> {
        // TODO old version
        let opengl = OpenGL::V4_5;

        let mut window: glutin_window::GlutinWindow =
            piston::WindowSettings::new("crust-8", [640, 320])
                .graphics_api(opengl)
                .exit_on_esc(true)
                .build()
                .unwrap();

        let mut gl = GlGraphics::new(opengl);

        let mut events = piston::Events::new(piston::EventSettings::new());

        while let Some(e) = events.next(&mut window) {
            // TODO can probably be smarter about not duplicating these checks
            // TODO look at the press and release implementations to see the underlying
            piston::input::PressEvent::press(&e, |button| {
                if let Some(key) = button.map_key() {
                    self.headless.lock().unwrap().key_buffer[key as usize] = true;

                    // TODO This is so sloppy
                    let maybe_locked_mutex = self.key_interrupt.lock();
                    let mut locked_mutex = maybe_locked_mutex.unwrap();
                    let maybe_sender: &mut Option<Sender<Key>> = locked_mutex.deref_mut();

                    if let Some(sender) = maybe_sender {
                        sender.send(key).unwrap();
                        *maybe_sender = None;
                    }
                }
            });
            piston::input::ReleaseEvent::release(&e, |button| {
                if let Some(key) = button.map_key() {
                    self.headless.lock().unwrap().key_buffer[key as usize] = false;
                }
            });

            if let Some(args) = e.render_args() {
                self.render(&mut gl, &args)
            }
        }

        Ok(())
    }
}

impl Chip8IO for PistonGraphics {
    // TODO do we need result types for these
    fn clear(&mut self) {
        self.headless.lock().unwrap().clear();
    }

    fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        self.headless.lock().unwrap().draw(x, y, sprite)
    }

    fn read_key(&mut self, key: Key) -> bool {
        self.headless.lock().unwrap().read_key(key)
    }

    fn block_for_key(&mut self, sender: Sender<Key>) {
        *self.key_interrupt.lock().unwrap() = Some(sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_pixel() {
        assert_eq!(None, HeadlessIO::index_pixel(32, 63));
        assert_eq!(None, HeadlessIO::index_pixel(31, 64));
        assert_eq!(None, HeadlessIO::index_pixel(32, 64));

        assert_eq!(0, HeadlessIO::index_pixel(0, 0).unwrap());
        assert_eq!(5, HeadlessIO::index_pixel(5, 0).unwrap());
        assert_eq!(63, HeadlessIO::index_pixel(63, 0).unwrap());
        assert_eq!(64, HeadlessIO::index_pixel(0, 1).unwrap());
        assert_eq!(140, HeadlessIO::index_pixel(12, 2).unwrap());
        assert_eq!(2047, HeadlessIO::index_pixel(63, 31).unwrap());
    }

    #[test]
    fn test_pixel_operations() {
        let mut graphics = HeadlessIO::new();

        // Should be off to start
        assert_eq!(Some(false), graphics.get_pixel(6, 1));
        graphics.video_buffer[70] = true;

        // Looking up the pixel by coordinate should be true now
        assert_eq!(Some(true), graphics.get_pixel(6, 1));

        // Flipping the pixel should return true because it was flipped off
        assert_eq!(true, graphics.flip_pixel(6, 1));

        // And the underlying pixel should now be false
        assert_eq!(Some(false), graphics.get_pixel(6, 1));

        // Flipping it back on should return false because nothing was turned off
        assert_eq!(false, graphics.flip_pixel(6, 1));
        assert_eq!(true, graphics.flip_pixel(6, 1));

        assert_eq!([false; 2048], graphics.video_buffer);
    }

    #[test]
    fn test_headless_graphics() {
        let mut graphics = HeadlessIO::new();

        // There should be 2048 boolean cells in the graphics buffer
        assert_eq!([false; 2048], graphics.video_buffer);

        // Drawing an empty sprite should not affect the buffer or indicate a flip
        let flipped = graphics.draw(0, 0, &[]);
        assert_eq!(false, flipped);
        assert_eq!([false; 2048], graphics.video_buffer);

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
        assert_eq!(pixels_aa, graphics.video_buffer[0..8]);
        assert_eq!(pixels_55, graphics.video_buffer[64..72]);
        assert_eq!(pixels_aa, graphics.video_buffer[128..136]);
        assert_eq!(pixels_55, graphics.video_buffer[192..200]);

        // Check a few things outside the draw area
        assert_eq!(pixels_00, graphics.video_buffer[8..16]);
        assert_eq!(pixels_00, graphics.video_buffer[200..208]);

        // Draw the checkerboard's inverse
        let flipped = graphics.draw(0, 0, &sprite_negative);
        assert_eq!(false, flipped);
        for y in 0..4 {
            assert_eq!(pixels_ff, graphics.video_buffer[y * 64..y * 64 + 8])
        }

        // Flip both checkerboards off again, should reset the board
        assert_eq!(true, graphics.draw(0, 0, &sprite_positive));
        assert_eq!(true, graphics.draw(0, 0, &sprite_negative));
        assert_eq!([false; 2048], graphics.video_buffer);
    }

    #[test]
    fn headless_graphics_out_of_bounds() {
        // An 8x8 all-black sprite
        let sprite = [0xFF; 8];

        let mut graphics = HeadlessIO::new();

        let flipped = graphics.draw(WIDTH_PX as u8 - 1, HEIGHT_PX as u8 - 1, &sprite);
        assert_eq!(false, flipped);

        let mut expected = [false; 2048];
        expected[2047] = true;

        assert_eq!(expected, graphics.video_buffer);
    }
}
