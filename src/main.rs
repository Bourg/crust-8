use crust_8::{graphics, machine, settings};
use std::thread;

fn main() {
    // Create two handles to the graphics implementation
    let window_graphics = graphics::PistonGraphics::new();
    let machine_graphics = window_graphics.clone();

    thread::spawn(move || {
        let mut machine = machine::Machine::new(
            machine_graphics,
            &settings::Settings {
                bit_shift_mode: settings::BitShiftMode::OneRegister,
            },
        );

        let completion_message = match machine.run() {
            Ok(()) => String::from("Machine completed successfully"),
            Err(e) => format!("Machine completed exceptionally: {:?}", e),
        };
        println!("{}", completion_message);
    });

    window_graphics.open_window().unwrap();
}
