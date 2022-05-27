use crust_8::graphics;
use crust_8::graphics::Draw;
use std::thread;
use std::time::Duration;

fn main() {
    let graphics = graphics::PistonGraphics::new();
    let mut thread_graphics = graphics.clone();

    thread::spawn(move || {
        let sprite = [0xAA, 0x55, 0xAA, 0xFF, 0x00, 0xFF, 0x55, 0xAA, 0x55];

        let mut x = 0;
        let mut y = 0;

        thread_graphics.draw(x, y, &sprite);
        loop {
            thread::sleep(Duration::from_secs(1));

            thread_graphics.draw(x, y, &sprite);
            x = x + 1;
            y = y + 1;
            thread_graphics.draw(x, y, &sprite);
        }
    });

    graphics.open_window().unwrap();
}
