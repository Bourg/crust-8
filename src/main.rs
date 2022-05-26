use crust_8::graphics;
use crust_8::graphics::EmulatorGraphics;

fn main() {
    let mut graphics = graphics::PistonGraphics::new();

    graphics.draw(
        5,
        5,
        &[0xAA, 0x55, 0xAA, 0xFF, 0x00, 0xFF, 0x55, 0xAA, 0x55],
    );

    graphics.open_window().unwrap();
}
