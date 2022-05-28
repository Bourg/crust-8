use crust_8::instruction::Instruction;
use Instruction::*;

pub fn get() -> Vec<Instruction> {
    vec![
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
    ]
}
