use crust_8::{graphics, machine, settings, timer};
use std::{env, fs, thread, time};

fn main() {
    // TODO error handling, more isolated CLI logic
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
            Some(time::Duration::from_millis(100)),
            machine_graphics,
            rand::thread_rng(),
            // TODO use a wall timer
            timer::InstructionTimer::new(),
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
