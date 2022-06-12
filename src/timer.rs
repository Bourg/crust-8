use std::time::{Duration, Instant};

pub trait Timer {
    fn should_tick(&mut self) -> bool;
}

// Timer implementation based on number of instructions executed
// Good for tests that run without clock
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

pub struct WallTimer {
    last_check: Instant,
    ticked: Duration,
}

impl WallTimer {
    // Chip8 timers tick at 60Hz
    // 16667 microseconds is the most precise approximation of 1/60th of a second
    const TICK_DURATION: Duration = Duration::from_micros(16667);

    pub fn new() -> WallTimer {
        WallTimer {
            last_check: Instant::now(),
            ticked: Duration::ZERO,
        }
    }

    fn should_tick_internal(&mut self, now: Instant) -> bool {
        let mut should_tick = false;

        if let Some(elapsed) = now.checked_duration_since(self.last_check) {
            self.ticked += elapsed;

            // If the ticker has crossed the duration of one tick
            if self.ticked >= WallTimer::TICK_DURATION {
                should_tick = true;

                let ticked_micros = self.ticked.as_micros();
                self.ticked = Duration::from_micros(
                    (ticked_micros % WallTimer::TICK_DURATION.as_micros()) as u64,
                )
            }
        }

        self.last_check = now;
        should_tick
    }
}

impl Timer for WallTimer {
    fn should_tick(&mut self) -> bool {
        self.should_tick_internal(Instant::now())
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

    #[test]
    fn wall_timer() {
        let mut timer = WallTimer::new();
        let now = Instant::now();

        // Have to test with injected instants so the test isn't dependent on system speed

        // An immediate tick should not roll over
        assert_eq!(false, timer.should_tick_internal(now));

        // 10,000 later should still not roll over
        let now = now + Duration::from_micros(10000);
        assert_eq!(false, timer.should_tick_internal(now));

        // Another 6,666 later is just a single micro from rolling over
        let now = now + Duration::from_micros(6666);
        assert_eq!(false, timer.should_tick_internal(now));

        // Another 6,668 should roll the timer over and leave a residual 6,667
        let now = now + Duration::from_micros(6668);
        assert!(timer.should_tick_internal(now));

        // Another 10,000 should perfectly roll the timer over to 0
        let now = now + Duration::from_micros(10000);
        assert!(timer.should_tick_internal(now));

        // A tick that is larger than the time should cause one tick
        let now = now + Duration::from_micros(16667 * 10 + 5555);
        assert!(timer.should_tick_internal(now));
        let now = now + Duration::from_micros(10000);
        assert_eq!(false, timer.should_tick_internal(now));

        // Should still be able to roll over normally
        let now = now + Duration::from_micros(1112);
        assert!(timer.should_tick_internal(now));
    }

    #[test]
    fn wall_timer_poisoned() {
        let mut timer = WallTimer::new();
        let now = Instant::now();

        // Suppose the internal timer ends up a full week in the future
        let the_future = now + Duration::from_secs(60 * 60 * 24 * 7);
        timer.last_check = the_future;

        // The timer should not tick
        assert_eq!(false, timer.should_tick_internal(now));

        // Given another cycle, it should tick normally
        assert!(timer.should_tick_internal(now + Duration::from_micros(16667)));
    }
}
