use crate::{apu::samples::SamplesMutex, clock::Clock, mapped::Mapped, CLOCK};
use device::Audio;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

pub mod device;
pub mod samples;

pub struct ApuInner<D: Audio> {
    _phantom: PhantomData<D>,

    sample: u64,

    pub(crate) ch0: Option<f64>,
    pub(crate) ch1: Option<f64>,
    pub(crate) ch2: Option<f64>,
    pub(crate) ch3: Option<f64>,

    // Sound Channel 1 - Tone & Sweep
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    // Sound Channel 2 - Tone
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    // Sound Channel 3 - Wave Output
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    wave_ram: [u8; 0x10],
    // Sound Channel 4 - Noise
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    // Sound Control Registers
    nr50: u8,
    // Bit 7 - Output sound 4 to SO2 terminal
    // Bit 6 - Output sound 3 to SO2 terminal
    // Bit 5 - Output sound 2 to SO2 terminal
    // Bit 4 - Output sound 1 to SO2 terminal
    // Bit 3 - Output sound 4 to SO1 terminal
    // Bit 2 - Output sound 3 to SO1 terminal
    // Bit 1 - Output sound 2 to SO1 terminal
    // Bit 0 - Output sound 1 to SO1 terminal
    nr51: u8,
    // Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
    // Bit 3 - Sound 4 ON flag (Read Only)
    // Bit 2 - Sound 3 ON flag (Read Only)
    // Bit 1 - Sound 2 ON flag (Read Only)
    // Bit 0 - Sound 1 ON flag (Read Only)
    nr52: u8,
}

// FIXME don't inline so much (channels 1 & 2 share some behaviour)
impl<D: Audio> ApuInner<D> {
    pub fn step(&mut self, cycles: u64) {}

    // clear APU registers except NR52's high bit
    fn power_off(&mut self) {
        self.nr10 = 0;
        self.nr11 = 0;
        self.nr12 = 0;
        self.nr13 = 0;
        self.nr14 = 0;

        self.nr21 = 0;
        self.nr22 = 0;
        self.nr23 = 0;
        self.nr24 = 0;

        self.nr30 = 0;
        self.nr31 = 0;
        self.nr32 = 0;
        self.nr33 = 0;
        self.nr34 = 0;

        self.nr41 = 0;
        self.nr42 = 0;
        self.nr43 = 0;
        self.nr44 = 0;

        self.nr50 = 0;
        self.nr51 = 0;
        self.nr52 &= 0x80;
    }
}

pub struct Apu<D: Audio> {
    inner: Arc<Mutex<ApuInner<D>>>,
}

impl<D: Audio> Default for Apu<D> {
    fn default() -> Self {
        let inner = ApuInner {
            _phantom: PhantomData,
            sample: 0,

            ch0: None,
            ch1: None,
            ch2: None,
            ch3: None,

            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,

            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,

            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            wave_ram: [0; 0x10],

            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,

            nr50: 0,
            nr51: 0,
            nr52: 0,
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl<D: Audio> Apu<D> {
    /// Return audio samples iterator.
    pub fn samples(&self) -> SamplesMutex<D> {
        SamplesMutex::new(&self.inner)
    }

    pub fn lock(&self) -> MutexGuard<ApuInner<D>> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

//
// - APU registers always have some bits set when read back.
// - Wave memory can be read back freely.
// - When powered off, registers are cleared, except high bit of NR52.
// - While off, register writes are ignored, but not reads.
// - Wave RAM is always readable and writable, and unaffected by power.
impl<D: Audio> Mapped for Apu<D> {
    fn read(&self, addr: u16) -> u8 {
        let apu = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        match addr {
            0xff10 => apu.nr10,
            0xff11 => apu.nr11,
            0xff12 => apu.nr12,
            0xff13 => apu.nr13,
            0xff14 => apu.nr14,

            0xff16 => apu.nr21,
            0xff17 => apu.nr22,
            0xff18 => apu.nr23,
            0xff19 => apu.nr24,

            0xff1a => apu.nr30,
            0xff1b => apu.nr31,
            0xff1c => apu.nr32,
            0xff1d => apu.nr33,
            0xff1e => apu.nr34,
            0xff30..=0xff3f => apu.wave_ram[addr as usize - 0xff30],

            0xff20 => apu.nr41,
            0xff21 => apu.nr42,
            0xff22 => apu.nr43,
            0xff23 => apu.nr44,

            0xff24 => apu.nr50,
            0xff25 => apu.nr51,

            // TODO
            0xff26 => apu.nr52 & 0x80,
            0xff27..=0xff2f => panic!(), // unused
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let mut apu = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if apu.nr52 & 0x80 != 0 {
            match addr {
                // Channel 1 sweep
                0xff10 => apu.nr10 = data,
                0xff11 => apu.nr11 = data,
                0xff12 => apu.nr12 = data,
                0xff13 => apu.nr13 = data,
                0xff14 => {
                    apu.nr14 = data & 0xc7;

                    if apu.nr14 & 0x80 != 0 {}
                }

                // Channel 2 - Tone
                0xff16 => apu.nr21 = data,
                0xff17 => apu.nr22 = data,
                0xff18 => apu.nr23 = data,
                0xff19 => {
                    apu.nr24 = data & 0xc7;

                    if apu.nr24 & 0x80 != 0 {}
                }

                // Channel 3 - Wave RAM
                0xff1a => apu.nr30 = data,
                0xff1b => apu.nr31 = data,
                0xff1c => apu.nr32 = data,
                0xff1d => apu.nr33 = data,
                0xff1e => {
                    apu.nr34 = data;

                    if apu.nr34 & 0x80 != 0 {}
                }
                0xff30..=0xff3f => { /* Handled below */ }

                // Channel 4 - Noise
                0xff20 => apu.nr41 = data,
                0xff21 => apu.nr42 = data,
                0xff22 => apu.nr43 = data,
                0xff23 => {
                    apu.nr44 = data;

                    if apu.nr44 & 0x80 != 0 {}
                }

                0xff24 => apu.nr50 = data,
                0xff25 => apu.nr51 = data,

                0xff26 => { /* Handled below */ }
                0xff27..=0xff2f => { /* Unused */ }
                _ => panic!(),
            }
        }

        // Wave RAM writes are unaffected by power status
        if let 0xff30..=0xff3f = addr {
            // let f = data >> 4;
            // let s = data & 0xf;
            // for _ in 0..f {
            //     print!("-");
            // }
            // println!("*");
            // for _ in 0..s {
            //     print!("-");
            // }
            // println!("*");
            apu.wave_ram[addr as usize - 0xff30] = data;
        }
        if addr == 0xff3f {
            // println!("===");
        }

        // Enable / Disable sound entirely
        if addr == 0xff26 {
            apu.nr52 &= 0x7f;
            apu.nr52 |= data & 0x80;

            if apu.nr52 & 0x80 == 0 {
                apu.power_off();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{apu::Apu, mapped::Mapped};

    #[test]
    fn wave_ram() {
        let mut apu = Apu::<()>::default();

        let wave = &[
            0x01_u8, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xcd, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];

        for (i, w) in wave.iter().copied().enumerate() {
            apu.write(0xff30 + i as u16, w)
        }

        for (i, w) in wave.iter().copied().enumerate() {
            assert_eq!(w, apu.read(0xff30 + i as u16));
        }
    }
}
