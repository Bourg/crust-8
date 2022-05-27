use crust_8::instruction::Instruction::*;
use crust_8::{graphics, instruction, machine, settings};
use std::thread;

fn main() {
    // TODO load the program from disk
    let program: &[instruction::Instruction] = &[
        // 0: x position
        // 1: y position
        // 2: Constant 5 for shifting
        // I: Sprite address
        StoreXNN {
            register: 0,
            value: 1,
        },
        StoreXY {
            target: 1,
            source: 0,
        },
        StoreXNN {
            register: 2,
            value: 5,
        },
        StoreNNN { value: 0 },
        // Draw
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        // Line break for hex
        StoreXNN {
            register: 0,
            value: 1,
        },
        AddXNN {
            register: 1,
            value: 6,
        },
        // Hex
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
        DrawXYN {
            x_register: 0,
            y_register: 1,
            bytes: 5,
        },
        AddXY {
            target: 0,
            source: 2,
        },
        AddIX { register: 2 },
    ];

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

        machine.load_program(program);

        let completion_message = match machine.run() {
            Ok(()) => String::from("Machine completed successfully"),
            Err(e) => format!("Machine completed exceptionally: {:?}", e),
        };
        println!("{}", completion_message);
    });

    window_graphics.open_window().unwrap();
}
