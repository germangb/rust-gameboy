use crate::{apu::samples::SamplesMutex, map::Mapped};
use device::AudioDevice;
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

/// Audio format spec.
pub mod device;
/// APU sample iterator.
pub mod samples;

const LEN_CLOCK: u64 = 256;
const VOL_CLOCK: u64 = 64;
const SWEEP_CLOCK: u64 = 128;

struct ToneChannel {
    len: Option<u64>,
}

struct WaveChannel {
    len: Option<u64>,
    sample: u64,
}

struct NoiseChannel {
    len: Option<u64>,
    lfsr: u16,
}

struct ApuInner<D: AudioDevice> {
    _phantom: PhantomData<D>,
    // clocked at whatever frequency in D
    sample: u64,

    ch0: Option<ToneChannel>,
    ch1: Option<ToneChannel>,
    ch2: Option<WaveChannel>,
    ch3: Option<NoiseChannel>,

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
    // sample channel 0 (sweep tone)
    // applies frequency sweeping, volume envelope, and duration
    fn channel0(&mut self) -> Option<f64> {
        if self.len_elapsed(0) {
            self.ch0 = None;
            return None;
        }

        let mut freq = self.freq(0);

        if self.ch0.is_some() {
            // frequency sweep
            let period = u64::from(self.nr10 >> 4) & 0x7;
            if period > 0 && self.sample % (period * D::sample_rate() / SWEEP_CLOCK) == 0 {
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
                    self.ch0 = None;
                    return None;
                }
            }

            let square = square_wave::<D>(self.sample, freq, self.nr11);

            let vol = self.vol_envelope(0);

            Some(square * vol)
        } else {
            None
        }
    }

    // sample channel 1 (tone)
    // applies volume envelope, and duration
    fn channel1(&mut self) -> Option<f64> {
        if self.len_elapsed(1) {
            self.ch1 = None;
            return None;
        }

        if self.ch1.is_some() {
            let freq = self.freq(1);
            let vol = self.vol_envelope(1);
            let square = square_wave::<D>(self.sample, freq, self.nr21);
            Some(square * vol)
        } else {
            None
        }
    }

    // sample channel 3 (wave ram)
    // updates wave ram pointer
    fn channel2(&mut self) -> Option<f64> {
        if self.nr30 & 0x80 == 0 {
            return None;
        }

        if self.len_elapsed(2) {
            self.ch2 = None;
            return None;
        }

        let freq = self.freq(2);

        if let Some(WaveChannel { ref mut sample, .. }) = self.ch2.as_mut() {
            // frequency timer
            let freq = (2048 - freq) * 2;
            if self.sample % (D::sample_rate() / freq) == 0 {
                *sample += 1;
                *sample %= 32;
            }

            // fetch sample and apply volume
            let mut wave_sample = self.wave_ram[*sample as usize / 2];
            if *sample % 2 == 0 {
                wave_sample >>= 4
            };
            wave_sample &= 0xf;
            let wave_sample = (wave_sample as f64 / 15_f64) * 0.5 + 0.5;
            let vol = match (self.nr32 >> 5) & 0x3 {
                0 => 0.0,
                1 => 1.0,
                2 => 0.5,
                3 => 0.25,
                _ => panic!(),
            };

            Some(wave_sample * vol)
        } else {
            None
        }
    }

    // sample channel 3 (noise)
    // applies volume envelope and updates LFSR
    fn ch3(&mut self) -> Option<f64> {
        if self.len_elapsed(3) {
            self.ch3 = None;
            return None;
        }

        if let Some(NoiseChannel { ref mut lfsr, .. }) = self.ch3.as_mut() {
            // update lfsr
            let shift = u64::from(self.nr43 >> 4);
            let freq = match self.nr43 & 0x7 {
                0 => 8,
                n => u64::from(n) * 16,
            } << shift;
            let period = D::sample_rate() / freq.min(D::sample_rate());

            // FIXME this is wrong, but I can't get it to output what it should. I'm
            // probably misunderstanding how this is supposed to work
            if self.nr43 & 0x8 != 0 {
                if self.sample % period == 0 {
                    let l0 = *lfsr & 0x1;
                    *lfsr >>= 1;
                    let l1 = *lfsr & 0x1;
                    let l6 = (l0 ^ l1) << 6;
                    *lfsr &= 0x3f;
                    *lfsr |= l6;
                }
            } else {
                let l0 = *lfsr & 0x1;
                *lfsr >>= 1;
                let l1 = *lfsr & 0x1;
                let l14 = (l0 ^ l1) << 14;
                *lfsr &= 0x3fff;
                *lfsr |= l14;
            }

            let amp = (*lfsr & 0x1) as f64 * 2.0 - 1.0;
            let vol = self.vol_envelope(3);

            Some(amp * vol)
        } else {
            None
        }
    }

    #[rustfmt::skip]
    fn vol_envelope(&mut self, ch: usize) -> f64 {
        let nrx2 = match ch {
            0 => &mut self.nr12,
            1 => &mut self.nr22,
            2 => panic!(),
            3 => &mut self.nr42,
            _ => panic!(),
        };

        let mut vol = u64::from(*nrx2 >> 4);
        let period = u64::from(*nrx2 & 0x7) * D::sample_rate() / VOL_CLOCK;
        if period != 0 && self.sample % period == 0 {
            match (*nrx2 >> 3) & 0x1 {
                0 => if vol > 0x0 { vol -= 1 }
                1 => if vol < 0xf { vol += 1 }
                _ => panic!(),
            }

            *nrx2 &= 0x0f;
            *nrx2 |= (vol as u8) << 4;
        }

        vol as f64 / 15_f64
    }

    // computed frequency from the NRx3 & NRx4 registers
    // which are common to challens 0..=2
    fn freq(&self, ch: usize) -> u64 {
        let (nrx3, nrx4) = match ch {
            0 => (self.nr13, self.nr14),
            1 => (self.nr23, self.nr24),
            2 => (self.nr33, self.nr34),
            3 => panic!(),
            _ => panic!(),
        };
        u64::from(nrx3) | (u64::from(nrx4 & 0x7) << 8)
    }

    // updates length timer on a given channel
    // returns true if the timer has elapsed and therefore the channel has to be
    // disabled
    fn len_elapsed(&mut self, ch: usize) -> bool {
        let mut len = match ch {
            0 => self.ch0.as_mut().and_then(|c| c.len.as_mut()),
            1 => self.ch1.as_mut().and_then(|c| c.len.as_mut()),
            2 => self.ch2.as_mut().and_then(|c| c.len.as_mut()),
            3 => self.ch3.as_mut().and_then(|c| c.len.as_mut()),
            _ => panic!(),
        };

        if let Some(ref mut len) = len {
            let period = D::sample_rate() / LEN_CLOCK;

            if self.sample % period == 0 {
                **len -= 1
            }

            **len == 0
        } else {
            false
        }
    }

    fn power_off(&mut self) {
        self.ch0 = None;
        self.ch1 = None;
        self.ch2 = None;
        self.ch3 = None;

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

fn lock<D: AudioDevice>(mutex: &Mutex<ApuInner<D>>) -> MutexGuard<ApuInner<D>> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn square_wave<D: AudioDevice>(sample: u64, freq: u64, nrx1: u8) -> f64 {
    let period = D::sample_rate() * (2048 - freq) / 131_072;
    if period != 0 {
        let sample = sample % period;
        let duty = u64::from(nrx1 >> 6);
        if duty == 0b00 && sample < period * 125 / 1000
            || duty == 0b01 && sample < period / 4
            || duty == 0b10 && sample < period / 2
            || duty == 0b11 && sample < period * 2 / 3
        {
            1.0
        } else {
            -1.0
        }
    } else {
        // FIXME crashes on tetris, v-Rally (period = 0)
        0.0
    }
}

pub struct Apu<D: AudioDevice> {
    inner: Arc<Mutex<ApuInner<D>>>,
}

impl<D: AudioDevice> Default for Apu<D> {
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

impl<D: AudioDevice> Apu<D> {
    /// Return audio samples iterator.
    pub fn samples(&self) -> SamplesMutex<D> {
        SamplesMutex::new(&self.inner)
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
        let apu = lock(&self.inner);

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
                if apu.ch0.is_some() { data |= 0x1; }
                if apu.ch1.is_some() { data |= 0x2; }
                if apu.ch2.is_some() { data |= 0x4; }
                if apu.ch3.is_some() { data |= 0x8; }
                data
            }
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let mut apu = lock(&self.inner);

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
                            Some(64 - u64::from(apu.nr11 & 0x3f))
                        } else {
                            None
                        };
                        apu.ch0 = Some(ToneChannel { len: timer });
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
                            Some(64 - u64::from(apu.nr11 & 0x3f))
                        } else {
                            None
                        };
                        apu.ch1 = Some(ToneChannel { len: timer });
                    }
                }

                // Channel 3 - Wave RAM
                0xff1a => apu.nr30 = data,
                0xff1b => apu.nr31 = data,
                0xff1c => apu.nr32 = data,
                0xff1d => apu.nr33 = data,
                0xff1e => {
                    apu.nr34 = data;

                    if apu.nr34 & 0x80 != 0 {
                        let timer = if apu.nr34 & 0x40 != 0 {
                            Some(256 - u64::from(apu.nr31))
                        } else {
                            None
                        };
                        let sample = apu.ch2.as_ref().map(|c| c.sample).unwrap_or(0);
                        apu.ch2 = Some(WaveChannel { len: timer, sample });
                    }
                }
                0xff30..=0xff3f => { /* Handled below */ }

                // Channel 4 - Noise
                0xff20 => apu.nr41 = data,
                0xff21 => apu.nr42 = data,
                0xff22 => apu.nr43 = data,
                0xff23 => {
                    apu.nr44 = data;

                    // println!("NR41 {:08b}", apu.nr41);
                    // println!("NR42 {:08b}", apu.nr42);
                    // println!("NR43 {:08b}", apu.nr43);
                    // println!("NR44 {:08b}", apu.nr44);
                    // println!("---");

                    if apu.nr44 & 0x80 != 0 {
                        let timer = if apu.nr44 & 0x40 != 0 {
                            Some(64 - u64::from(apu.nr41 & 0x3f))
                        } else {
                            None
                        };
                        apu.ch3 = Some(NoiseChannel {
                            len: timer,
                            lfsr: 0x7fff,
                        });
                    }
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
        if addr == 0xff3f {
            // println!("===");
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
