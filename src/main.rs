use crust_8::io::piston_io;
use crust_8::{cli, machine, settings, timer};
use std::sync::mpsc;
use std::{fs, thread, time};

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    let file = fs::File::open(cli.rom_path).unwrap();

    // Create two handles to the graphics implementation
    let window_io = piston_io::PistonIO::new(cli.color_scheme.into());
    let machine_io = window_io.clone();

    let settings = settings::Settings::default().with_clock_speed(settings::ClockSpeed::Limited {
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
