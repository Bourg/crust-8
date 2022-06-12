use crust_8::io::piston_io::PistonIO;
use crust_8::settings::ClockSpeed;
use crust_8::{machine, settings, timer};
use std::sync::mpsc;
use std::{env, fs, thread, time};

fn main() {
    // TODO error handling, more isolated CLI logic
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).unwrap();
    let file = fs::File::open(filename).unwrap();

    // Create two handles to the graphics implementation
    let window_io = PistonIO::new();
    let machine_io = window_io.clone();

    let settings = settings::Settings::default()
        .with_clock_speed(ClockSpeed::Limited {
            instruction_time: time::Duration::from_millis(2),
        })
        .with_on_unrecognized_instruction(settings::OnUnrecognizedInstruction::Skip);

    // TODO use a channel to send a ready message from the IO since it takes a while to init
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut machine = machine::Machine::new(
            machine_io,
            rand::thread_rng(),
            // TODO use a wall timer in real-time since 500Hz/60Hz is not an integer
            timer::InstructionTimer::new(),
            settings,
        );

        // TODO better error handling
        machine.load_program(file).unwrap();

        // Wait to get the ready message from the UI thread
        rx.recv().unwrap();

        let completion_message = match machine.run_program() {
            Ok(()) => String::from("Machine completed successfully"),
            Err(e) => format!("Machine completed exceptionally: {:?}", e),
        };
        println!("{}", completion_message);
    });

    // Open the window and post a ready message
    window_io.open_window(|| tx.send(()).unwrap());
}
