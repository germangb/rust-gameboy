use crate::{
    apu::sfx::{LenCounter, Noise, Source, Square, SquareSweep, Volume, VolumeEnv},
    dev::Device,
};
use std::sync::{Arc, Mutex};

mod sfx;

struct ApuInner {
    ch1: Option<Box<dyn Source>>,
    ch2: Option<Box<dyn Source>>,
    ch3: Option<Box<dyn Source>>,
    ch4: Option<Box<dyn Source>>,

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

impl ApuInner {
    fn power_off(&mut self) {
        self.ch1 = None;
        self.ch2 = None;
        self.ch3 = None;
        self.ch4 = None;

        // clear APU registers except NR52's high bit

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

unsafe impl Send for ApuInner {}

pub struct Apu {
    inner: Arc<Mutex<ApuInner>>,
}

impl Default for Apu {
    fn default() -> Self {
        let inner = ApuInner {
            ch1: None,
            ch2: None,
            ch3: None,
            ch4: None,

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

impl Apu {
    pub fn samples(&self) -> Samples {
        Samples {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Iterator of samples produced by the APU.
///
/// # Panics
/// The iterator panics if the APU lock gets poisoned.
pub struct Samples {
    inner: Arc<Mutex<ApuInner>>,
}

impl Iterator for Samples {
    type Item = [i16; 2];

    #[rustfmt::skip]
    fn next(&mut self) -> Option<Self::Item> {
        let mut apu = self.inner.lock().unwrap();

        let ch1 = apu.ch1.as_mut().and_then(|c| c.sample());
        let ch2 = apu.ch2.as_mut().and_then(|c| c.sample());
        let ch3 = apu.ch3.as_mut().and_then(|c| c.sample());
        let ch4 = apu.ch4.as_mut().and_then(|c| c.sample());

        // Handle ended channels
        // Unset trigger bits on each register
        if ch1.is_none() { apu.ch1 = None; apu.nr14 &= 0x7f; }
        if ch2.is_none() { apu.ch2 = None; apu.nr24 &= 0x7f; }
        if ch3.is_none() { apu.ch3 = None; apu.nr34 &= 0x7f; }
        if ch4.is_none() { apu.ch4 = None; apu.nr44 &= 0x7f; }

        // audio mixing
        let mut so: [i32; 2] = [0, 0];
        let mut count: [i32; 2] = [0, 0];

        let nr51 = apu.nr51;
        for (ch, sample) in ch1.into_iter().chain(ch2).chain(ch3).chain(ch4).enumerate() {
            let so1_bit = 1 << (ch as u8);
            let so2_bit = 1 << (4 + ch as u8);
            if nr51 & so1_bit != 0 {
                so[0] += sample as i32;
                count[0] += 1;
            }
            if nr51 & so2_bit != 0 {
                so[1] += sample as i32;
                count[1] += 1;
            }
        }

        if count[0] > 0 { so[0] /= count[0] }
        if count[1] > 0 { so[1] /= count[1] }

        Some([so[0] as i16, so[1] as i16])
    }
}

//
// - APU registers always have some bits set when read back.
// - Wave memory can be read back freely.
// - When powered off, registers are cleared, except high bit of NR52.
// - While off, register writes are ignored, but not reads.
// - Wave RAM is always readable and writable, and unaffected by power.
impl Device for Apu {
    fn read(&self, addr: u16) -> u8 {
        let apu = self.inner.lock().expect("Error locking APU");
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
            #[rustfmt::skip]
            0xff26 => {
                let mut data = apu.nr52;
                if apu.ch1.is_some() { data |= 0x1; }
                if apu.ch2.is_some() { data |= 0x2; }
                if apu.ch3.is_some() { data |= 0x4; }
                if apu.ch4.is_some() { data |= 0x3; }
                data
            }
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let mut apu = self.inner.lock().expect("Error locking APU");

        if apu.nr52 & 0x80 != 0 {
            match addr {
                // Channel 1 sweep
                0xff10 => apu.nr10 = data,
                0xff11 => apu.nr11 = data,
                0xff12 => apu.nr12 = data,
                0xff13 => apu.nr13 = data,
                0xff14 => {
                    apu.nr14 = data & 0xc7;

                    if apu.nr14 & 0x80 != 0 {
                        let tone = SquareSweep::new(apu.nr10, apu.nr11, apu.nr13, apu.nr14);
                        apu.ch1 = Some(apply_len_envelope(tone, apu.nr11, apu.nr12, apu.nr14));
                    }
                }

                // Channel 2 - Tone
                0xff16 => apu.nr21 = data,
                0xff17 => apu.nr22 = data,
                0xff18 => apu.nr23 = data,
                0xff19 => {
                    apu.nr24 = data & 0xc7;

                    if apu.nr24 & 0x80 != 0 {
                        let tone = Square::new(apu.nr21, apu.nr23, apu.nr24);
                        apu.ch2 = Some(apply_len_envelope(tone, apu.nr21, apu.nr22, apu.nr24));
                    }
                }

                // Channel 3 - Wave RAM
                0xff1a => apu.nr30 = data,
                0xff1b => apu.nr31 = data,
                0xff1c => apu.nr32 = data,
                0xff1d => apu.nr33 = data,
                0xff1e => apu.nr34 = data,
                0xff30..=0xff3f => { /* below */ }

                // Channel 4 - Noise
                0xff20 => apu.nr41 = data,
                0xff21 => apu.nr42 = data,
                0xff22 => apu.nr43 = data,
                0xff23 => {
                    apu.nr44 = data;

                    if apu.nr44 & 0x80 != 0 {
                        let tone =
                            apply_len_envelope(Noise::new(apu.nr43), apu.nr41, apu.nr42, apu.nr44);
                        apu.ch4 = Some(tone);
                    }
                }

                0xff24 => apu.nr50 = data,
                0xff25 => apu.nr51 = data,

                0xff26 => { /* below */ }
                _ => panic!(),
            }
        }

        // Wave RAM writes are unaffected by power status
        if let 0xff30..=0xff3f = addr {
            apu.wave_ram[addr as usize - 0xff30] = data;
        }

        // If your GB programs don't use sound then write 00h to this register to save
        // 16% or more on GB power consumption. Disabeling the sound controller
        // by clearing Bit 7 destroys the contents of all sound registers. Also,
        // it is not possible to access any sound registers (execpt FF26) while
        // the sound controller is disabled.
        //
        // Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
        // Bit 3 - Sound 4 ON flag (Read Only)
        // Bit 2 - Sound 3 ON flag (Read Only)
        // Bit 1 - Sound 2 ON flag (Read Only)
        // Bit 0 - Sound 1 ON flag (Read Only)
        //
        // Bits 0-3 of this register are read only status bits, writing to these bits
        // does NOT enable/disable sound. The flags get set when sound output is
        // restarted by setting the Initial flag (Bit 7 in NR14-NR44), the flag
        // remains set until the sound length has expired (if enabled). A volume
        // envelopes which has decreased to zero volume will NOT cause the sound
        // flag to go off.
        if addr == 0xff26 {
            apu.nr52 &= 0x7f;
            apu.nr52 |= data & 0x80;

            // when powered off, all registers are cleared except NR52.7
            if apu.nr52 & 0x80 == 0 {
                apu.power_off();
            }
        }
    }
}

fn apply_len_envelope(
    source: impl Source + 'static,
    mut nr_1: u8,
    nr_2: u8,
    nr_4: u8,
) -> Box<dyn Source> {
    nr_1 &= 0x3f;

    let len_enabled = nr_4 & 0x40 != 0;
    let env_enabled = nr_2 & 0x7 != 0;

    let wave: Box<dyn Source> = if env_enabled && len_enabled {
        Box::new(LenCounter::new(VolumeEnv::new(source, nr_2), nr_1))
    } else if env_enabled {
        Box::new(VolumeEnv::new(source, nr_2))
    } else if len_enabled {
        Box::new(LenCounter::new(Volume::new(source, nr_2), nr_1))
    } else {
        Box::new(Volume::new(source, nr_2))
    };

    wave
}
