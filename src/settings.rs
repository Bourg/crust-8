#[derive(Copy, Clone)]
pub enum BitShiftMode {
    // This is the documented bit shift mode
    TwoRegister,
    // This is a bug that many programs depend on (source register is ignored)
    OneRegister,
}

#[derive(Copy, Clone)]
pub enum OnUnrecognizedInstruction {
    // Halt execution exceptionally
    Halt,

    // Skip the instruction and continue execution
    Skip,
}

#[derive(Copy, Clone)]
pub struct Settings {
    pub bit_shift_mode: BitShiftMode,
    pub on_unrecognized_instruction: OnUnrecognizedInstruction,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            bit_shift_mode: BitShiftMode::OneRegister,
            on_unrecognized_instruction: OnUnrecognizedInstruction::Halt,
        }
    }
}
