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

        self.sample += 1;

        let samp = self.sample % peri;

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

        if self.sample == SAMPLE_RATE / 64 {
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
    fn sample(&mut self) -> Option<i16> {
        None
    }
}

pub struct Wave {
    sample: u64,
    wave: [u8; 0x10],
    wave_sample: usize,
    freq: u64,
    // length timer
    time: Option<u64>,
    // volume code:
    //  00 - 0% (no sound)
    //  01 - 100%
    //  10 - 50%
    //  11 - 25%
    volume: u64,
}

impl Wave {
    pub fn new(nr31: u8, nr32: u8, nr33: u8, nr34: u8, wave: [u8; 0x10]) -> Self {
        Self {
            sample: 0,
            wave,
            wave_sample: 0,
            freq: u64::from(nr33) | (u64::from(nr34 & 0x7) << 8),
            time: None,
            volume: u64::from(nr32 >> 5) & 0x3,
        }
    }
}

impl Source for Wave {
    fn sample(&mut self) -> Option<i16> {
        self.sample += 1;

        let freq_period = 2 * (2048 - self.freq);
        if self.sample == SAMPLE_RATE / freq_period {
            self.sample = 0;
            self.wave_sample += 1;
            self.wave_sample %= 32;
        }

        let shift = match self.volume {
            0 => 4,
            1 => 0,
            2 => 1,
            3 => 3,
            _ => panic!(),
        };

        let mut sample = self.wave[self.wave_sample / 2];
        if self.wave_sample % 2 == 0 {
            sample >>= 4;
        }
        sample &= 0xf;
        sample >>= shift;

        let mut sample = sample as f64;
        sample /= 0xf as f64;
        sample *= 2.0;
        sample -= 1.0;
        sample *= 0x7fff as f64;

        let sample = sample as i16;
        //println!("{} | {}", sample, self.freq);

        Some(sample as i16)
    }
}
