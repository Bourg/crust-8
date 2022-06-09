use crate::memory;

const FLAG_REGISTER: u8 = 0xF;
const STACK_SIZE: u8 = 16;

pub struct Registers {
    pub v: [u8; 16],
    pub i: memory::Address,
    pub pc: memory::Address,
    pub sp: u8,
    pub dt: u8,
    pub st: u8,

    // TODO not sure if registers is the best place for this
    // It's related to the PC register, but also memory
    // Maybe memory + registers should be combined?
    pub stack: [memory::Address; STACK_SIZE as usize],
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
            stack: [0; 16],
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

    pub fn stack_call(&mut self, address: memory::Address) {
        // Put the PC on top of the stack
        self.stack[self.sp as usize] = self.pc;

        // Increment the stack pointer
        self.sp = self.sp + 1 % STACK_SIZE;

        // Jump to the called routine
        self.pc = address & 0xFFF;

        // Do not advance the PC for calls
        // The called address is the first instruction of the routine
    }

    pub fn stack_return(&mut self) {
        // Decrement the stack pointer and wrap if needed
        self.sp = if self.sp == 0 {
            STACK_SIZE - 1
        } else {
            self.sp - 1
        };

        // Set the program counter to the top of the stack
        self.pc = self.stack[self.sp as usize];

        // Returning always advances the PC
        // The address on top of the stack will always be that of the CALL
        self.advance_pc();
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

    #[test]
    fn test_call_return() {
        let mut registers = Registers::new();

        let start_pc = registers.pc;

        registers.stack_call(0x500);
        assert_eq!(start_pc, registers.stack[0]);
        assert_eq!(1, registers.sp);
        assert_eq!(0x500, registers.pc);

        registers.stack_return();
        assert_eq!(start_pc, registers.stack[0]);
        assert_eq!(0, registers.sp);
        assert_eq!(start_pc + 2, registers.pc);
    }
}
