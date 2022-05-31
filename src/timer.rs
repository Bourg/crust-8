pub trait Timer {
    fn should_tick(&mut self) -> bool;
}

// Timer implementation based on number of instructions executed
// Good for tests that run without clock
// TODO maybe combine the clock speed mode with the timers since they are related
pub struct InstructionTimer {
    counter: u8,
}

impl InstructionTimer {
    const INSTRUCTIONS_PER_TICK: u8 = 8;

    pub fn new() -> InstructionTimer {
        InstructionTimer {
            counter: InstructionTimer::INSTRUCTIONS_PER_TICK,
        }
    }
}

impl Timer for InstructionTimer {
    fn should_tick(&mut self) -> bool {
        let should_tick = self.counter == 1;

        self.counter = if should_tick {
            InstructionTimer::INSTRUCTIONS_PER_TICK
        } else {
            self.counter - 1
        };

        should_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_timer() {
        let mut timer = InstructionTimer::new();

        for _ in 0..5 {
            // Run a cycle of 7 non-ticks followed 1 tick
            for _ in 0..7 {
                assert!(!timer.should_tick());
            }
            assert!(timer.should_tick());
        }
        assert!(!timer.should_tick());
    }
}
