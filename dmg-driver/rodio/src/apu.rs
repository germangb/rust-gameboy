use dmg_lib::apu::{device::AudioDevice, Samples};
use rodio::Source;
use std::time::Duration;

pub type Depth = f32;

pub struct RodioSamples<D: AudioDevice<Sample = Depth>> {
    samples: Samples<D>,
}

impl<D: AudioDevice<Sample = Depth>> RodioSamples<D> {
    pub fn new(samples: Samples<D>) -> Self {
        Self { samples }
    }
}

impl<D: AudioDevice<Sample = Depth>> Iterator for RodioSamples<D> {
    type Item = Depth;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.next()
    }
}

impl<D: AudioDevice<Sample = Depth>> Source for RodioSamples<D> {
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
