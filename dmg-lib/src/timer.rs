use crate::{
    dev::Device,
    interrupts::{Flag, Interrupts},
    CLOCK,
};

pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    // Bit 2    - Timer Stop  (0=Stop, 1=Start)
    // Bits 1-0 - Input Clock Select
    //            00:   4096 Hz    (~4194 Hz SGB)
    //            01: 262144 Hz  (~268400 Hz SGB)
    //            10:  65536 Hz   (~67110 Hz SGB)
    //            11:  16384 Hz   (~16780 Hz SGB)
    tac: u8,
    div_cycles: u64,
    tima_cycles: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            div_cycles: 0,
            tima_cycles: 0,
        }
    }
}

impl Timer {
    pub(crate) fn step(&mut self, cycles: u64, int: &mut Interrupts) {
        // DIV counter
        self.div_cycles += cycles;
        let cycles_per_tick = CLOCK / 16_384 / 2;

        if self.div_cycles > cycles_per_tick {
            self.div_cycles %= cycles_per_tick;
            self.div = self.div.wrapping_add(1);
        }

        if self.tac & 0x4 == 0 {
            return;
        }

        // TIMA counter
        self.tima_cycles += cycles as u64;
        let cycles_per_tick = CLOCK / self.clock();

        if self.tima_cycles > cycles_per_tick {
            self.tima_cycles %= cycles_per_tick;
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0 {
                self.tima = self.tma;
                int.set(Flag::Timer);
            }
        }
    }

    fn clock(&self) -> u64 {
        match self.tac & 0x3 {
            0 => 4_096,
            1 => 262_144,
            2 => 65_536,
            3 => 16_384,
            _ => panic!(),
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
            0xff04 => {
                //self.div_cycles = 0;
                self.div = 0
            }
            0xff05 => self.tima = data,
            0xff06 => self.tma = data,
            0xff07 => self.tac = data,
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {}
