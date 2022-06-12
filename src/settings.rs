use std::time;

#[derive(Copy, Clone)]
pub enum BitShiftMode {
    // Bit shift the Y register and store the result in X
    // This is the documented bit shift mode
    TwoRegister,
    // Bit shift the X register and store the result in X, ignoring Y
    // This is a bug that many programs depend on (source register is ignored)
    OneRegister,
}

#[derive(Copy, Clone)]
pub enum ClockSpeed {
    Unlimited,
    Limited { instruction_time: time::Duration },
}

#[derive(Copy, Clone)]
pub enum MemoryMode {
    // Advance the I register on store and load instructions
    Advance,
    // Do not advance the I register on store and load instructions
    NoAdvance,
}

#[derive(Copy, Clone)]
pub struct Settings {
    pub bit_shift_mode: BitShiftMode,
    pub clock_speed: ClockSpeed,
    pub memory_mode: MemoryMode,
}

impl Settings {
    pub fn with_bit_shift_mode(mut self, bit_shift_mode: BitShiftMode) -> Self {
        self.bit_shift_mode = bit_shift_mode;
        self
    }

    pub fn with_clock_speed(mut self, clock_speed: ClockSpeed) -> Self {
        self.clock_speed = clock_speed;
        self
    }

    pub fn with_memory_mode(mut self, memory_mode: MemoryMode) -> Self {
        self.memory_mode = memory_mode;
        self
    }
}

impl Default for Settings {
    // Default settings are good for tests
    fn default() -> Self {
        Settings {
            bit_shift_mode: BitShiftMode::OneRegister,
            clock_speed: ClockSpeed::Unlimited,
            memory_mode: MemoryMode::NoAdvance,
        }
    }
}
