use crust_8::{graphics, machine, settings};
use std::{env, fs, thread, time};

fn main() {
    // TODO better error handling
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).unwrap();
    let file = fs::File::open(filename).unwrap();

    // Create two handles to the graphics implementation
    let window_graphics = graphics::PistonGraphics::new();
    let machine_graphics = window_graphics.clone();

    thread::spawn(move || {
        let mut machine = machine::Machine::new(
            // Clock speed is 500Hz, so 2ms/operation
            //Some(time::Duration::from_millis(2)),
            Some(time::Duration::from_millis(250)),
            machine_graphics,
            rand::thread_rng(),
            settings::Settings {
                bit_shift_mode: settings::BitShiftMode::OneRegister,
                on_unrecognized_instruction: settings::OnUnrecognizedInstruction::Skip,
            },
        );

        // TODO better error handling
        machine.load_program(file).unwrap();

        let completion_message = match machine.run_program() {
            Ok(()) => String::from("Machine completed successfully"),
            Err(e) => format!("Machine completed exceptionally: {:?}", e),
        };
        println!("{}", completion_message);
    });

    window_graphics.open_window().unwrap();
}
