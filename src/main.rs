mod test_programs;

use crust_8::{graphics, machine, settings};
use std::thread;

fn main() {
    // TODO load the program from disk
    let program = test_programs::default_sprites::get();

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

        machine.load_program(&program);

        let completion_message = match machine.run() {
            Ok(()) => String::from("Machine completed successfully"),
            Err(e) => format!("Machine completed exceptionally: {:?}", e),
        };
        println!("{}", completion_message);
    });

    window_graphics.open_window().unwrap();
}
