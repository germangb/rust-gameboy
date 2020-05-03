pub struct Clock {
    base: u64,
    freq: u64,
    tick: u64,
}

impl Clock {
    pub fn new(base: u64, freq: u64) -> Self {
        assert!(freq <= base);
        Self {
            base,
            freq,
            tick: 0,
        }
    }

    // Update the clock. Returns the number of clock cycles elapsed after the update
    // (normally 1).
    pub fn step(&mut self, cycles: u64) -> u64 {
        let cycles_tick = self.base / self.freq;
        self.tick += cycles;
        let clocks = self.tick / cycles_tick;
        self.tick %= cycles_tick;
        clocks
    }
}

#[cfg(test)]
mod test {
    use crate::clock::Clock;

    #[test]
    fn clock() {
        let mut clock = Clock::new(1000, 500);

        assert_eq!(0, clock.step(1));
        assert_eq!(1, clock.step(1));
        assert_eq!(0, clock.step(1));
        assert_eq!(1, clock.step(1));

        assert_eq!(2, clock.step(4));
        assert_eq!(1, clock.step(2));
        assert_eq!(0, clock.step(1));
        assert_eq!(1, clock.step(1));
    }
}
