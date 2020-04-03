use crate::{apu::device::Sample, map::Mapped};
use device::AudioDevice;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub mod device;

struct ToneChannel {
    // Loaded duration
    timer: Option<u64>,
    // NR11 FF11 DDLL LLLL Duty, Length load (64-L)
    nrx1: u8,
    // VVVV APPP Starting volume, Envelope add mode, period
    nrx2: u8,
}

struct ApuInner<D: AudioDevice> {
    _phantom: PhantomData<D>,
    // clocked at whatever frequency in D
    sample: u64,

    ch1: Option<ToneChannel>,
    ch2: Option<ToneChannel>,
    ch3: Option<()>,
    ch4: Option<()>,

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
impl<D: AudioDevice> ApuInner<D> {
    #[rustfmt::skip]
    fn ch1(&mut self) -> Option<f64> {
        if let Some(ToneChannel {
            ref mut timer,
            nrx1: ref mut nr11,
            nrx2: ref mut nr12,
        }) = self.ch1.as_mut()
        {
            // duration timer
            if let Some(ref mut timer) = timer {
                let period = D::sample_rate() / 256;
                if self.sample % period == 0 {
                    *timer -= 1;
                }

                if *timer == 0 {
                    self.ch1 = None;
                    return None;
                }
            }

            let mut freq = u64::from(self.nr13) | (u64::from(self.nr14 & 0x7) << 8);

            // frequency sweep
            let period = u64::from(self.nr10 >> 4) & 0x7;
            if period > 0 && self.sample % (period * D::sample_rate() / 128) == 0 {
                // Compute frequency and optionally negate value
                let shift = u64::from(self.nr10) & 0x7;
                let mut shadow = freq >> shift;
                if self.nr10 & 0x8 != 0 {
                    shadow = !shadow;
                }
                freq = freq.wrapping_add(shadow);
                self.nr13 = (freq & 0xff) as u8;
                self.nr14 &= !0x7;
                self.nr14 |= (freq >> 8) as u8;
                // The overflow check simply calculates the new frequency and if this is greater
                // than 2047, square 1 is disabled.
                if freq >= 2048 {
                    self.ch1 = None;
                    return None;
                }
            }

            let period = D::sample_rate() * (2048 - freq) / 131072;
            let sample = self.sample % period;
            let duty = u64::from(self.nr11 >> 6);
            let square = if duty == 0b00 && sample < period * 125 / 1000
                || duty == 0b01 && sample < period / 4
                || duty == 0b10 && sample < period / 2
                || duty == 0b11 && sample < period * 2 / 3
            { 1.0 } else { -1.0 };

            // volume envelope
            let mut vol = u64::from(*nr12 >> 4);
            let period = *nr12 & 0x7;
            if period != 0 {
                let period = u64::from(period) * D::sample_rate() / 64;
                if self.sample % period == 0 {
                    match (self.nr12 >> 3) & 0x1 {
                        0 => if vol > 0x0 { vol -= 1 }
                        1 => if vol < 0xf { vol += 1 }
                        _ => panic!(),
                    }
                    *nr12 &= 0x0f;
                    *nr12 |= (vol as u8) << 4;
                }
            }
            let vol = vol as f64 / (0xf as f64);

            Some(square * vol)
        } else {
            None
        }
    }

    #[rustfmt::skip]
    fn ch2(&mut self) -> Option<f64> {
        if let Some(ToneChannel {
            ref mut timer,
            nrx1: ref mut nr21,
            nrx2: ref mut nr22,
        }) = self.ch2.as_mut()
        {
            // duration timer
            if let Some(ref mut timer) = timer {
                let period = D::sample_rate() / 256;
                if self.sample % period == 0 {
                    *timer -= 1;
                }

                if *timer == 0 {
                    self.ch2 = None;
                    return None;
                }
            }

            let freq = u64::from(self.nr23) | (u64::from(self.nr24 & 0x7) << 8);

            let duty = u64::from(self.nr21 >> 6);
            let period = D::sample_rate() * (2048 - freq) / 131072;
            let sample = self.sample % period;
            let square = if duty == 0b00 && sample < period * 125 / 1000
                || duty == 0b01 && sample < period / 4
                || duty == 0b10 && sample < period / 2
                || duty == 0b11 && sample < period * 2 / 3
            { 1.0 } else { -1.0 };

            // volume envelope
            let mut vol = u64::from(*nr22 >> 4);
            let period = *nr22 & 0x7;
            if period != 0 {
                let period = u64::from(period) * D::sample_rate() / 64;
                if self.sample % period == 0 {
                    match (self.nr22 >> 3) & 0x1 {
                        0 => if vol > 0x0 { vol -= 1 }
                        1 => if vol < 0xf { vol += 1 }
                        _ => panic!(),
                    }
                    *nr22 &= 0x0f;
                    *nr22 |= (vol as u8) << 4;
                }
            }
            let vol = vol as f64 / (0xf as f64);

            Some(square * vol)
        } else {
            None
        }
    }

    #[rustfmt::skip]
    fn ch3(&mut self) -> Option<f64> {
        None
    }

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

unsafe impl<D: AudioDevice + Send> Send for ApuInner<D> {}

pub struct Apu<D: AudioDevice> {
    inner: Arc<Mutex<ApuInner<D>>>,
}

impl<D: AudioDevice> Apu<D> {
    pub fn new() -> Self {
        let inner = ApuInner {
            _phantom: PhantomData,
            sample: 0,

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

    pub fn samples(&self) -> Samples<D> {
        Samples {
            inner: Arc::clone(&self.inner),
            buffer: None,
        }
    }
}

#[derive(Clone, Copy)]
enum Buffer<D: AudioDevice> {
    Two([D::Sample; 2]),
    One(D::Sample),
}

/// Iterator of samples produced by the APU.
///
/// # Panics
/// The iterator panics if the APU lock gets poisoned.
pub struct Samples<D: AudioDevice> {
    inner: Arc<Mutex<ApuInner<D>>>,
    buffer: Option<Buffer<D>>,
}

impl<D: AudioDevice> Samples<D> {
    /// Loads the nest sample into the buffer
    fn load(&mut self) {
        let mut apu = self.inner.lock().unwrap();

        apu.sample += 1;
        let ch1 = apu.ch1();
        let ch2 = apu.ch2();
        let ch3 = apu.ch3();
        // let ch4 = apu.ch4.as_mut().and_then(|c| c.sample());

        // audio mixing
        let mut so: [f64; 2] = [0.0; 2];
        let mut count: [u32; 2] = [0; 2];

        let nr51 = apu.nr51;
        for (ch, sample) in ch1
            .into_iter()
            .chain(ch2)
            .chain(ch3)
            // .chain(ch4)
            .enumerate()
        {
            let so1_bit = 1 << (ch as u8);
            let so2_bit = 1 << (4 + ch as u8);
            if nr51 & so1_bit != 0 {
                so[0] += sample;
                count[0] += 1;
            }
            if nr51 & so2_bit != 0 {
                so[1] += sample;
                count[1] += 1;
            }
        }

        if count[0] > 0 {
            so[0] /= count[0] as f64;
        }
        if count[1] > 0 {
            so[1] /= count[1] as f64;
        }

        let max: f64 = D::Sample::max().as_f64();
        let min: f64 = D::Sample::min().as_f64();
        let l = clamp(so[0] * 0.5 + 0.5, 0.0, 1.0);
        let l = min * (1.0 - l) + max * l;
        let r = so[1] * 0.5 + 0.5;
        let r = min * (1.0 - r) + max * r;

        self.buffer = if D::mono() {
            let mix = D::Sample::from_f64((l + r) / 2.0);
            Some(Buffer::One(mix))
        } else {
            let l = D::Sample::from_f64(l);
            let r = D::Sample::from_f64(r);
            Some(Buffer::Two([l, r]))
        };
    }
}

fn clamp(n: f64, min: f64, max: f64) -> f64 {
    if n > max {
        max
    } else if n < min {
        min
    } else {
        n
    }
}

impl<D: AudioDevice> Iterator for Samples<D> {
    type Item = D::Sample;

    #[rustfmt::skip]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.buffer {
                None => self.load(),
                Some(Buffer::Two([ch0, ch1])) => {
                    self.buffer = Some(Buffer::One(ch1));
                    return Some(ch0);
                },
                Some(Buffer::One(ch)) => {
                    self.buffer = None;
                    return Some(ch);
                }
            }
        }
    }
}

//
// - APU registers always have some bits set when read back.
// - Wave memory can be read back freely.
// - When powered off, registers are cleared, except high bit of NR52.
// - While off, register writes are ignored, but not reads.
// - Wave RAM is always readable and writable, and unaffected by power.
impl<D: AudioDevice> Mapped for Apu<D> {
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
                        let timer = if apu.nr14 & 0x40 != 0 {
                            Some(u64::from(apu.nr11 & 0x3f))
                        } else {
                            None
                        };
                        apu.ch1 = Some(ToneChannel {
                            timer,
                            nrx1: apu.nr11,
                            nrx2: apu.nr12,
                        });
                    }
                }

                // Channel 2 - Tone
                0xff16 => apu.nr21 = data,
                0xff17 => apu.nr22 = data,
                0xff18 => apu.nr23 = data,
                0xff19 => {
                    apu.nr24 = data & 0xc7;

                    if apu.nr24 & 0x80 != 0 {
                        let timer = if apu.nr14 & 0x40 != 0 {
                            Some(u64::from(apu.nr11 & 0x3f))
                        } else {
                            None
                        };
                        apu.ch2 = Some(ToneChannel {
                            timer,
                            nrx1: apu.nr21,
                            nrx2: apu.nr22,
                        });
                    }
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
                }

                0xff24 => apu.nr50 = data,
                0xff25 => apu.nr51 = data,

                0xff26 => { /* Handled below */ }
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
        // if addr == 0xff3f {
        //     println!("===");
        // }

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

#[cfg(test)]
mod test {
    use crate::{apu::Apu, map::Mapped};

    #[test]
    fn wave_ram() {
        let mut apu = Apu::default();

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
