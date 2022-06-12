use crate::io::chip8_io::Chip8IO;
use crate::io::graphics::{GraphicsBuffer, SpriteData};
use crate::io::input::{Key as Chip8Key, Keypad, MapKey};
use glutin_window::OpenGL;
use opengl_graphics::GlGraphics;
use piston::input::Key as PistonKey;
use piston::{ButtonArgs, ButtonEvent, ButtonState, Event, RenderArgs, RenderEvent};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};

// TODO actually need to play audio when ST > 1

// TODO I'm really not super happy with the ownership structure for window graphics
// TODO maybe have two structs, and only one non-cloneable is aware of the window?
#[derive(Clone)]
pub struct PistonIO {
    internal: Arc<Mutex<PistonIOInternal>>,
}

struct PistonIOInternal {
    graphics_buffer: GraphicsBuffer,
    keypad: Keypad,
    interrupt_channel: Option<Sender<Chip8Key>>,
}

impl PistonIOInternal {
    pub fn new() -> Self {
        PistonIOInternal {
            graphics_buffer: GraphicsBuffer::new(),
            keypad: Keypad::new(),
            interrupt_channel: None,
        }
    }

    fn handle_button_event(&mut self, args: ButtonArgs) {
        match (args.state, args.button.map_key()) {
            (ButtonState::Press, Some(key)) => {
                self.keypad.press(key);
                if let Some(channel) = &mut self.interrupt_channel {
                    // Ignore a failed send
                    channel.send(key).unwrap_or(());
                    self.interrupt_channel = None;
                }
            }
            (ButtonState::Release, Some(key)) => {
                self.keypad.release(&key);
            }
            _ => {}
        }
    }

    pub fn handle_event(&mut self, e: Event, gl: &mut GlGraphics) {
        if let Some(button_args) = e.button_args() {
            self.handle_button_event(button_args);
        }

        if let Some(args) = e.render_args() {
            self.render(gl, &args);
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        gl.draw(args.viewport(), |c, gl| {
            graphics::clear([1.0, 1.0, 1.0, 1.0], gl);

            // TODO iterator over the pixels?
            for y in 0..self.graphics_buffer.height() {
                for x in 0..self.graphics_buffer.width() {
                    let pixel = self.graphics_buffer.get_pixel(x as u8, y as u8);

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
        });
    }
}

impl Chip8IO for PistonIO {
    fn clear(&mut self) {
        self.internal.lock().unwrap().graphics_buffer.clear();
    }

    fn draw(&mut self, x: u8, y: u8, sprite: &SpriteData) -> bool {
        self.internal
            .lock()
            .unwrap()
            .graphics_buffer
            .draw(x, y, sprite)
    }

    fn key_pressed(&mut self, key: Chip8Key) -> bool {
        self.internal.lock().unwrap().keypad.is_pressed(&key)
    }

    fn block_for_key(&mut self) -> Option<Chip8Key> {
        let (tx, rx) = mpsc::channel();

        self.internal.lock().unwrap().interrupt_channel = Some(tx);
        rx.recv().ok()
    }
}

impl PistonIO {
    pub fn new() -> Self {
        let internal = PistonIOInternal::new();

        PistonIO {
            internal: Arc::new(Mutex::new(internal)),
        }
    }

    pub fn open_window<F>(self, on_ready: F)
    where
        F: FnOnce(),
    {
        let opengl = OpenGL::V4_5;

        let mut window: glutin_window::GlutinWindow =
            piston::WindowSettings::new("crust-8", [640, 320])
                .graphics_api(opengl)
                .exit_on_esc(true)
                .build()
                .unwrap();

        // TODO passing a lot around, maybe can be smarter about object structure
        let mut gl = GlGraphics::new(opengl);

        let mut events = piston::Events::new(piston::EventSettings::new());

        if let Some(_) = events.next(&mut window) {
            on_ready();
        }

        while let Some(e) = events.next(&mut window) {
            // TODO can probably be smarter about not duplicating these checks
            // TODO look at the press and release implementations to see the underlying
            let mut internal = self.internal.lock().unwrap();
            internal.handle_event(e, &mut gl);
        }
    }
}

impl MapKey for piston::input::Button {
    fn map_key(&self) -> Option<Chip8Key> {
        match self {
            piston::Button::Keyboard(piston_key) => piston_key.map_key(),
            _ => None,
        }
    }
}

// TODO move
impl MapKey for PistonKey {
    fn map_key(&self) -> Option<Chip8Key> {
        match self {
            PistonKey::D1 => Some(Chip8Key::D1),
            PistonKey::D2 => Some(Chip8Key::D2),
            PistonKey::D3 => Some(Chip8Key::D3),
            PistonKey::D4 => Some(Chip8Key::C),
            PistonKey::Q => Some(Chip8Key::D4),
            PistonKey::W => Some(Chip8Key::D5),
            PistonKey::E => Some(Chip8Key::D6),
            PistonKey::R => Some(Chip8Key::D),
            PistonKey::A => Some(Chip8Key::D7),
            PistonKey::S => Some(Chip8Key::D8),
            PistonKey::D => Some(Chip8Key::D9),
            PistonKey::F => Some(Chip8Key::E),
            PistonKey::Z => Some(Chip8Key::A),
            PistonKey::X => Some(Chip8Key::D0),
            PistonKey::C => Some(Chip8Key::B),
            PistonKey::V => Some(Chip8Key::F),
            _ => None,
        }
    }
}
