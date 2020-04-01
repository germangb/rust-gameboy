use crate::{dev::Device, CLOCK};
use std::sync::mpsc::{Receiver, Sender, SyncSender};

const SAMPLING: u64 = 44100;
const CYCLES_PER_SAMPLE: u64 = CLOCK / SAMPLING;
const BUFFER_SIZE: u64 = 1024 * 4;

pub trait AudioOutput {
    fn queue(&mut self, samples: &[i16]);
}

impl AudioOutput for () {
    fn queue(&mut self, _: &[i16]) {}
}

fn square_wave(sample: u64, freq: u64, pattern: u8) -> i64 {
    //let hz = 131072.0 / (2048.0 - freq as f64);
    let period = SAMPLING * (2048 - freq) / 131072;
    let sample = sample % period;

    if pattern == 0b00 && sample < period * 125 / 1000
        || pattern == 0b01 && sample < period / 4
        || pattern == 0b10 && sample < period / 2
        || pattern == 0b11 && sample < period * 2 / 3
    {
        1
    } else {
        -1
    }
}

#[derive(Clone, Copy)]
struct Ch1 {
    sample_begin: u64,
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
}

#[derive(Clone, Copy)]
struct Ch2 {
    sample_begin: u64,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
}

#[derive(Clone, Copy)]
struct Ch4 {
    sample_begin: u64,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
}

#[allow(dead_code)]
pub struct Apu<A: AudioOutput> {
    cycles: u64,
    state: u64,

    out: A,
    out_sample: u64,

    // Buffer to hold a frame worth of samples (RATE / 60)
    buf: Box<[i16; BUFFER_SIZE as usize]>,
    buf_samples: u64,

    ch1: Option<Ch1>,
    ch2: Option<Ch2>,
    ch3: (),
    ch4: Option<Ch4>,

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

impl<A: AudioOutput> Apu<A> {
    pub fn with_audio(output: A) -> Self {
        Self {
            cycles: 0,
            state: 1,

            out: output,
            out_sample: 0,

            buf: Box::new([0; BUFFER_SIZE as usize]),
            buf_samples: 0,

            ch1: None,
            ch2: None,
            ch3: (),
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
        }
    }

    pub fn next_sample(&mut self) -> i16 {
        self.buf_samples = 0;
        while self.buf_samples != 1 {
            self.step(4);
        }
        self.buf[0]
    }

    pub fn step(&mut self, cycles: u64) {
        self.cycles += cycles;

        // generate new sample
        if self.cycles >= CYCLES_PER_SAMPLE {
            self.cycles %= CYCLES_PER_SAMPLE;
            self.out_sample += 1;

            let mut sample1 = None;
            let mut sample2: Option<i16> = None;
            let mut sample3: Option<i16> = None;
            let mut sample4: Option<i16> = None;

            if let Some(Ch1 {
                sample_begin,
                nr10,
                nr11,
                nr12,
                nr13,
                nr14,
            }) = self.ch1
            {
                // TODO length
                let max_volume = u64::from(nr12 >> 4);
                let volume_sweep_samples = u64::from(nr12 & 0x7) * max_volume * SAMPLING / 64;
                let sample_end = if volume_sweep_samples == 0 {
                    sample_begin + SAMPLING * 10
                } else {
                    sample_begin + volume_sweep_samples
                };

                // TODO sweep
                let mut freq = u64::from(nr14 & 0x7) << 8 | u64::from(nr13);

                let vol_len = (sample_end - sample_begin) as i64;
                let vol_sample = (self.out_sample - sample_begin) as i64;
                let vol = if nr12 & 0x8 != 0 {
                    (vol_sample).min(vol_len)
                } else {
                    (vol_len - vol_sample).max(0)
                };
                let vol = (max_volume as i64) * 32000 * vol / i64::from(vol_len) / 15;
                let wave_pattern = nr11 >> 6;
                let sample_wave = square_wave(self.out_sample, freq, wave_pattern) * vol;

                sample1 = Some(sample_wave as i16);

                if self.out_sample > sample_end {
                    self.ch1 = None;
                }
            }

            if let Some(Ch2 {
                sample_begin,
                nr21,
                nr22,
                nr23,
                nr24,
            }) = self.ch2
            {
                // TODO length

                let max_volume = u64::from(nr22 >> 4);
                let volume_sweep_samples = u64::from(nr22 & 0x7) * max_volume * SAMPLING / 64;
                let sample_end = if volume_sweep_samples == 0 {
                    sample_begin + SAMPLING * 10
                } else {
                    sample_begin + volume_sweep_samples
                };

                let freq = u64::from(nr24 & 0x7) << 8 | u64::from(nr23);

                let vol_len = (sample_end - sample_begin) as i64;
                let vol_sample = (self.out_sample - sample_begin) as i64;
                let vol = if nr22 & 0x8 != 0 {
                    (vol_sample).min(vol_len)
                } else {
                    (vol_len - vol_sample).max(0)
                };
                let vol = (max_volume as i64) * 32000 * vol / i64::from(vol_len) / 15;
                let wave_pattern = nr21 >> 6;
                let sample_wave = square_wave(self.out_sample, freq, wave_pattern) * vol;

                sample2 = Some(sample_wave as i16);

                if self.out_sample > sample_end {
                    self.ch2 = None;
                }
            }

            let mut mix: i64 = 0;
            let mut count = 0;

            for sample in sample1
                .into_iter()
                .chain(sample2)
                .chain(sample3)
                .chain(sample4)
            {
                count += 1;
                mix += i64::from(sample);
            }

            if count > 0 {
                let mixed = (mix / count) as i16;
                self.buf[self.buf_samples as usize] = mixed;
            } else {
                self.buf[self.buf_samples as usize] = 0;
            }

            self.buf_samples += 1;
        }
    }

    pub fn flush(&mut self) {
        let over = self.state * 90;
        self.state = 1 - self.state;

        let frame_samples = SAMPLING / 60 + over;
        while self.buf_samples < frame_samples {
            self.step(4);
        }

        self.out.queue(&self.buf[..frame_samples as usize]);
        self.buf_samples = 0;
    }
}

impl<A: AudioOutput> Device for Apu<A> {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff10 => self.nr10,
            0xff11 => self.nr11,
            0xff12 => self.nr12,
            0xff13 => self.nr13,
            0xff14 => self.nr14,

            0xff16 => self.nr21,
            0xff17 => self.nr22,
            0xff18 => self.nr23,
            0xff19 => self.nr24,

            0xff1a => self.nr30,
            0xff1b => self.nr31,
            0xff1c => self.nr32,
            0xff1d => self.nr33,
            0xff1e => self.nr34,
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30],

            0xff20 => self.nr41,
            0xff21 => self.nr42,
            0xff22 => self.nr43,
            0xff23 => self.nr44,

            0xff24 => self.nr50,
            0xff25 => self.nr51,
            #[rustfmt::skip]
            0xff26 => {
                let mut data = self.nr52;
                if self.ch1.is_some()  { data |= 0x1; }
                if self.ch2.is_some()  { data |= 0x2; }
                //if self.ch3.is_some()  { data |= 0x4; }
                if self.ch4.is_some()  { data |= 0x3; }
                data
            }
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // Channel 1 sweep
            0xff10 => {
                self.nr10 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR10 = {:08b}", data);
            }
            0xff11 => {
                self.nr11 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR11 = {:08b}", data);
            }
            0xff12 => {
                self.nr12 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR12 = {:08b}", data);
            }
            0xff13 => {
                self.nr13 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR13 = {:08b}", data);
            }
            0xff14 => {
                self.nr14 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR14 = {:08b}", data);

                self.ch1 = Some(Ch1 {
                    sample_begin: self.out_sample,
                    nr10: self.nr10,
                    nr11: self.nr11,
                    nr12: self.nr12,
                    nr13: self.nr13,
                    nr14: self.nr14,
                });
            }

            // Channel 2 - Tone
            0xff16 => {
                self.nr21 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR21 = {:08b}", data);
            }
            0xff17 => {
                self.nr22 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR22 = {:08b}", data);
            }
            0xff18 => {
                self.nr23 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR23 = {:08b}", data);
            }
            0xff19 => {
                self.nr24 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR24 = {:08b}", data);

                self.ch2 = Some(Ch2 {
                    sample_begin: self.out_sample,
                    nr21: self.nr21,
                    nr22: self.nr22,
                    nr23: self.nr23,
                    nr24: self.nr24,
                });
            }

            // Channel 3 - Wave RAM
            0xff1a => self.nr30 = data,
            0xff1b => self.nr31 = data,
            0xff1c => self.nr32 = data,
            0xff1d => self.nr33 = data,
            0xff1e => self.nr34 = data,
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30] = data,

            // Channel 4 - Noise
            0xff20 => self.nr41 = data,
            0xff21 => self.nr42 = data,
            0xff22 => self.nr43 = data,
            0xff23 => {
                self.nr44 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR44 = {:08b}", data);

                self.ch4 = Some(Ch4 {
                    sample_begin: self.out_sample,
                    nr41: self.nr41,
                    nr42: self.nr42,
                    nr43: self.nr43,
                    nr44: self.nr44,
                });
            }

            0xff24 => self.nr50 = data,
            0xff25 => self.nr51 = data,

            // If your GB programs don't use sound then write 00h to this register to save 16% or
            // more on GB power consumption. Disabeling the sound controller by clearing Bit 7
            // destroys the contents of all sound registers. Also, it is not possible to access any
            // sound registers (execpt FF26) while the sound controller is disabled.
            //
            // Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
            // Bit 3 - Sound 4 ON flag (Read Only)
            // Bit 2 - Sound 3 ON flag (Read Only)
            // Bit 1 - Sound 2 ON flag (Read Only)
            // Bit 0 - Sound 1 ON flag (Read Only)
            //
            // Bits 0-3 of this register are read only status bits, writing to these bits does NOT
            // enable/disable sound. The flags get set when sound output is restarted by setting the
            // Initial flag (Bit 7 in NR14-NR44), the flag remains set until the sound length has
            // expired (if enabled). A volume envelopes which has decreased to zero volume will NOT
            // cause the sound flag to go off.
            0xff26 => {
                self.nr52 = data & 0x80;

                #[cfg(feature = "logging")]
                log::info!(target: "apu", "NR52 = {:08b}", data);
            }
            _ => panic!(),
        }
    }
}
