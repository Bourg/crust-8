use clap::Parser;
use crust_8::io::piston_io;
use crust_8::{cli, machine, random, settings, timer};
use std::error;
use std::sync::mpsc;
use std::{thread, time};

fn main() -> Result<(), Box<dyn error::Error>> {
    let cli = cli::Cli::parse();

    // Create two handles to the graphics implementation
    let window_io = piston_io::PistonIO::new(cli.color_scheme.into());
    let machine_io = window_io.clone();

    let settings = settings::Settings::default().with_clock_speed(settings::ClockSpeed::Limited {
        instruction_time: time::Duration::from_millis(2),
    });

    let mut machine = machine::Machine::new(
        machine_io,
        random::ThreadRandomSource,
        timer::WallTimer::new(),
        settings,
    );

    machine.load_program(cli.rom)?;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
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

    Ok(())
}
