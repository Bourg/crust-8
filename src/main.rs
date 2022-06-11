use crust_8::io::graphics;
use crust_8::settings::ClockSpeed;
use crust_8::{machine, settings, timer};
use std::{env, fs, thread, time};

fn main() {
    // TODO error handling, more isolated CLI logic
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).unwrap();
    let file = fs::File::open(filename).unwrap();

    // Create two handles to the graphics implementation
    let window_graphics = graphics::PistonGraphics::new();
    let machine_graphics = window_graphics.clone();

    let settings = settings::Settings::default()
        .with_clock_speed(ClockSpeed::Limited {
            instruction_time: time::Duration::from_millis(2),
        })
        .with_on_unrecognized_instruction(settings::OnUnrecognizedInstruction::Skip);

    // TODO use a channel to send a ready message from the IO since it takes a while to init

    thread::spawn(move || {
        let mut machine = machine::Machine::new(
            machine_graphics,
            rand::thread_rng(),
            // TODO use a wall timer in real-time since 500Hz/60Hz is not an integer
            timer::InstructionTimer::new(),
            settings,
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
