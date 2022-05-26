#[derive(Copy, Clone)]
pub enum BitShiftMode {
    // This is the documented bit shift mode
    TwoRegister,
    // This is a bug that many programs depend on (source register is ignored)
    OneRegister,
}

pub struct Settings {
    pub bit_shift_mode: BitShiftMode,
}
