use crate::memory;

const FLAG_REGISTER: u8 = 0xF;

pub struct Registers {
    pub v: [u8; 16],
    pub i: memory::Address,
    pub pc: memory::Address,
    pub sp: u8,
    pub dt: u8,
    pub st: u8,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            dt: 0,
            st: 0,
        }
    }

    pub fn set_register(&mut self, index: u8, value: u8) {
        self.v[index as usize] = value;
    }

    pub fn get_register(&self, index: u8) -> u8 {
        self.v[index as usize]
    }

    pub fn set_flag(&mut self, value: u8) {
        self.set_register(FLAG_REGISTER, value);
    }

    pub fn get_flag(&self) -> u8 {
        self.get_register(FLAG_REGISTER)
    }

    pub fn advance_pc(&mut self) {
        // Instructions are 2-byte aligned, so advance by 2
        self.pc += 2;
    }

    pub fn tick_timers(&mut self) {
        self.dt = if self.dt >= 1 { self.dt - 1 } else { self.dt };
        self.st = if self.st >= 1 { self.st - 1 } else { self.st };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_pc() {
        let mut registers = Registers::new();

        assert_eq!(0x200, registers.pc);
        registers.advance_pc();
        assert_eq!(0x202, registers.pc);

        registers.pc = 0x544;
        registers.advance_pc();
        assert_eq!(0x546, registers.pc);
        registers.advance_pc();
        assert_eq!(0x548, registers.pc);
    }

    #[test]
    fn tick_timers() {
        let mut registers = Registers::new();

        registers.dt = 4;
        registers.st = 3;

        registers.tick_timers();
        assert_eq!(3, registers.dt);
        assert_eq!(2, registers.st);

        registers.tick_timers();
        assert_eq!(2, registers.dt);
        assert_eq!(1, registers.st);

        registers.tick_timers();
        assert_eq!(1, registers.dt);
        assert_eq!(0, registers.st);

        registers.tick_timers();
        assert_eq!(0, registers.dt);
        assert_eq!(0, registers.st);

        registers.tick_timers();
        assert_eq!(0, registers.dt);
        assert_eq!(0, registers.st);
    }
}
