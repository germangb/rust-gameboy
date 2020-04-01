use dmg_lib::apu::Samples;
use rodio::Source;
use std::time::Duration;

pub struct RodioSamples {
    samples: Samples,
    l: Option<i16>,
    r: Option<i16>,
}

impl RodioSamples {
    pub fn new(samples: Samples) -> Self {
        Self {
            samples,
            l: None,
            r: None,
        }
    }
}

impl Iterator for RodioSamples {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.r.is_none() {
            self.samples.next().into_iter().for_each(|[l, r]| {
                self.l = Some(l);
                self.r = Some(r);
            });
        }
        self.l.take().or_else(|| self.r.take())
    }
}

impl Source for RodioSamples {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
