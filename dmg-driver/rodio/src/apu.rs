use dmg_lib::apu::{device::AudioDevice, Samples};
use rodio::Source;
use std::time::Duration;

pub struct RodioSamples<D: AudioDevice> {
    samples: Samples<D>,
}

impl<D: AudioDevice> RodioSamples<D> {
    pub fn new(samples: Samples<D>) -> Self {
        Self { samples }
    }
}

impl<D> Iterator for RodioSamples<D>
where
    D: AudioDevice,
    D::Sample: rodio::Sample,
{
    type Item = D::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.next()
    }
}

impl<D> Source for RodioSamples<D>
where
    D: AudioDevice,
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
