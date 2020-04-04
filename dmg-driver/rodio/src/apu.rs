use dmg_lib::apu::{device::Audio, samples::SamplesMutex, Apu};
use rodio::Source;
use std::time::Duration;

pub struct RodioSamples<D: Audio> {
    samples: SamplesMutex<D>,
}

impl<D: Audio> RodioSamples<D> {
    pub fn new(apu: &Apu<D>) -> Self {
        Self {
            samples: apu.samples(),
        }
    }
}

impl<D> Iterator for RodioSamples<D>
where
    D: Audio,
    D::Sample: rodio::Sample,
{
    type Item = D::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.lock().next()
    }
}

impl<D> Source for RodioSamples<D>
where
    D: Audio,
    D::Sample: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        if D::mono() {
            1
        } else {
            2
        }
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
