use std::time::Instant;

const SAMPLE_RATE: u64 = 44_100;

pub trait Source {
    // Returns the next sample.
    // Return None when the channel is stopped.
    fn sample(&mut self) -> Option<i16>;
}

#[derive(Debug)]
pub struct Volume<S: Source> {
    source: S,
    volume: u64,
}

impl<S: Source> Volume<S> {
    pub fn new(source: S, nr_2: u8) -> Self {
        Volume {
            source,
            volume: u64::from(nr_2 >> 4),
        }
    }
}

impl<S: Source> Source for Volume<S> {
    fn sample(&mut self) -> Option<i16> {
        self.source.sample().map(|s| {
            let amp = (0x7fff * self.volume / 0xf) as i16;
            s * amp
        })
    }
}

#[derive(Debug)]
pub struct Square {
    sample: u64,
    freq: u64,
    patt: u64,
}

impl Square {
    pub fn new(nr_1: u8, nr_3: u8, nr_4: u8) -> Self {
        Self {
            sample: 0,
            freq: u64::from(nr_3) | u64::from(nr_4 & 0x7) << 8,
            patt: 2,
        }
    }

    pub fn from_freq_and_patt(freq: u64, patt: u64) -> Self {
        Self {
            sample: 0,
            freq,
            patt,
        }
    }
}

impl Source for Square {
    fn sample(&mut self) -> Option<i16> {
        let patt = self.patt & 3;
        let peri = SAMPLE_RATE * (2048 - self.freq) / 131_072;

        let samp = self.sample % peri;

        self.sample += 1;

        if patt == 0b00 && samp < peri * 125 / 1000
            || patt == 0b01 && samp < peri / 4
            || patt == 0b10 && samp < peri / 2
            || patt == 0b11 && samp < peri * 2 / 3
        {
            Some(1)
        } else {
            Some(-1)
        }
    }
}

#[derive(Debug)]
pub struct SquareSweep {
    sample: u64,
    square: Square,
    // Square pattern and freq
    freq: u64,
    patt: u64,
    // Bit 6-4 - Sweep Time
    // Bit 3   - Sweep Increase/Decrease
    //     0: Addition    (frequency increases)
    //     1: Subtraction (frequency decreases)
    mode: u64,
    time: u64,
    // Bit 2-0 - Number of sweep shift (n: 0-7)
    shift: u64,
}

impl SquareSweep {
    pub fn new(nr10: u8, nr11: u8, nr13: u8, nr14: u8) -> Self {
        let freq = u64::from(nr13) | u64::from(nr14 & 0x7) << 8;
        let patt = u64::from(nr11 >> 6);

        Self {
            sample: 0,
            square: Square::from_freq_and_patt(freq, patt),
            freq,
            patt,
            mode: u64::from((nr10 >> 3) & 0x1),
            time: u64::from((nr10 >> 4) & 0x7),
            shift: u64::from(nr10 & 0x7),
        }
    }
}

impl Source for SquareSweep {
    fn sample(&mut self) -> Option<i16> {
        if self.freq >= 0x7ff {
            return None;
        }

        self.sample += 1;

        if self.sample == self.time * SAMPLE_RATE / 128 {
            // During a trigger event, several things occur:
            //
            // Square 1's frequency is copied to the shadow register.
            // The sweep timer is reloaded.
            // The internal enabled flag is set if either the sweep period or shift are
            // non-zero, cleared otherwise. If the sweep shift is non-zero,
            // frequency calculation and the overflow check are performed immediately.
            //
            // Frequency calculation consists of taking the value in the frequency shadow
            // register, shifting it right by sweep shift, optionally negating the value,
            // and summing this with the frequency shadow register to produce a new
            // frequency. What is done with this new frequency depends on the context.
            //
            // The overflow check simply calculates the new frequency and if this is greater
            // than 2047, square 1 is disabled.
            //
            // The sweep timer is clocked at 128 Hz by the frame sequencer. When it
            // generates a clock and the sweep's internal enabled flag is set and the sweep
            // period is not zero, a new frequency is calculated and the overflow check is
            // performed. If the new frequency is 2047 or less and the sweep shift is not
            // zero, this new frequency is written back to the shadow frequency and square
            // 1's frequency in NR13 and NR14, then frequency calculation and overflow check
            // are run AGAIN immediately using this new value, but this second new frequency
            // is not written back.
            //
            // Square 1's frequency can be modified via NR13 and NR14 while sweep is active,
            // but the shadow frequency won't be affected so the next time the sweep updates
            // the channel's frequency this modification will be lost.
            let shadow = self.freq >> 1;

            match self.mode {
                0 => self.freq += shadow,
                1 => self.freq -= shadow,
                _ => panic!(),
            }

            if self.freq > 0x7ff {
                return None;
            }

            self.square.freq = self.freq;
            self.sample = 0;
        }

        self.square.sample()
    }
}

#[derive(Debug)]
pub struct VolumeEnv<S: Source> {
    sample: u64,
    source: S,
    // Initial volume
    // Envelope:
    //  0 - decreasing
    //  1 - increasing
    init: u64,
    mode: u64,
    // envelope period
    // in 64Hz ticks
    sweep: u64,
}

impl<S: Source> VolumeEnv<S> {
    pub fn new(source: S, nr_2: u8) -> Self {
        Self {
            sample: 0,
            source,
            init: u64::from(nr_2 >> 4),
            mode: u64::from((nr_2 >> 3) & 0x1),
            sweep: u64::from(nr_2 & 0x7),
        }
    }
}

impl<S: Source> Source for VolumeEnv<S> {
    #[rustfmt::skip]
    fn sample(&mut self) -> Option<i16> {
        self.sample += 1;

        if self.sample == self.sweep * SAMPLE_RATE / 64 {
            self.sample = 0;
            match self.mode {
                0 => if self.init > 0x0 { self.init -= 1 },
                1 => if self.init < 0xf { self.init += 1 },
                _ => panic!(),
            }
        }

        let amp = (0x7fff * self.init / 0xf) as i16;
        self.source.sample().map(|s| s * amp)
    }
}

#[derive(Debug)]
pub struct LenCounter<S: Source> {
    sample: u64,
    source: S,
    // 256Hz length counter. When it drops to 0, the sound stops.
    len: u64,
}

impl<S: Source> LenCounter<S> {
    pub fn new(source: S, nr_1: u8) -> Self {
        Self {
            sample: 0,
            source,
            len: u64::from(nr_1 & 0x3f),
        }
    }
}

impl<S: Source> Source for LenCounter<S> {
    fn sample(&mut self) -> Option<i16> {
        if self.len == 0 {
            return None;
        }

        self.sample += 1;

        if self.sample == SAMPLE_RATE / 256 {
            self.sample = 0;
            self.len -= 1;
        }

        self.source.sample()
    }
}

pub struct Noise {}

impl Noise {
    pub fn new(nr43: u8) -> Self {
        Self {}
    }
}

impl Source for Noise {
    // TODO
    fn sample(&mut self) -> Option<i16> {
        Some(0)
    }
}
