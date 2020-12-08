use crate::{clock::Clock, device::Device, interrupt::Flag, CLOCK};

/// DMG timer emulation.
pub struct Timer {
    div: u8,
    div_clock: Clock,
    tima: u8,
    tma: u8,
    tac: u8,
    tima_clock: Clock,
    tima_int: Option<Flag>,
}

impl Default for Timer {
    fn default() -> Self {
        Self { div: 0,
               div_clock: Clock::new(CLOCK, 16_384),
               tima: 0,
               tma: 0,
               tac: 0,
               tima_int: None,
               tima_clock: Clock::new(0, 0) }
    }
}

impl Timer {
    pub fn step(&mut self, cycles: u64) {
        self.step_div(cycles);

        if self.tac & 0x4 != 0 {
            self.step_tima(cycles);
        }
    }

    pub(crate) fn take_timer_int(&mut self) -> Option<Flag> {
        self.tima_int.take()
    }

    fn step_div(&mut self, cycles: u64) {
        let div_cycles = self.div_clock.step(cycles);
        self.div = self.div.wrapping_add(div_cycles as u8);
    }

    fn step_tima(&mut self, cycles: u64) {
        for _ in 0..self.tima_clock.step(cycles) {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima = self.tma;
                self.tima_int = Some(Flag::Timer);
            }
        }
    }
}

impl Device for Timer {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => self.div,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff04 => self.div = 0,
            0xff05 => self.tima = data,
            0xff06 => self.tma = data,
            // Bit 2    - Timer Stop  (0=Stop, 1=Start)
            // Bits 1-0 - Input Clock Select
            //            00:   4096 Hz    (~4194 Hz SGB)
            //            01: 262144 Hz  (~268400 Hz SGB)
            //            10:  65536 Hz   (~67110 Hz SGB)
            //            11:  16384 Hz   (~16780 Hz SGB)
            0xff07 => {
                self.tac = data;
                let freq = match self.tac & 0x3 {
                    0 => 4_096,
                    1 => 262_144,
                    2 => 65_536,
                    3 => 16_384,
                    _ => panic!(),
                };
                self.tima_clock = Clock::new(CLOCK, freq);
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {}
