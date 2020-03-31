use dmg_lib::apu::AudioOutput;
use rodio::{buffer::SamplesBuffer, Sink};
use std::time::Duration;
// use std::time::{Instant, Duration};

pub struct RodioAudioOutput {
    sink: Sink,
}

impl RodioAudioOutput {
    pub fn new() -> Self {
        let device =
            rodio::default_output_device().expect("Error initializing Rodio output device");
        let sink = Sink::new(&device);
        sink.set_volume(0.2);

        Self { sink }
    }
}

impl AudioOutput for RodioAudioOutput {
    fn queue(&mut self, samples: &[i16]) {
        let buf = samples.to_vec();
        self.sink.append(SamplesBuffer::new(1, 44100, buf));
    }
}
