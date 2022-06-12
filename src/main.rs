use crust_8::io::piston_io::{PistonIO, WHITE_ON_BLACK};
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
    let window_io = PistonIO::new(WHITE_ON_BLACK);
    let machine_io = window_io.clone();

    let settings = settings::Settings::default().with_clock_speed(ClockSpeed::Limited {
        instruction_time: time::Duration::from_millis(2),
    });

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut machine = machine::Machine::new(
            machine_io,
            rand::thread_rng(),
            timer::WallTimer::new(),
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
