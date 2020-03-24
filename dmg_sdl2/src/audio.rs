use dmg_lib::apu::{AudioOutput, Sample};
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

pub struct Sdl2AudioOutput {
    channels: [AudioQueue<i16>; 1],
}

impl Sdl2AudioOutput {
    pub fn new(audio: &AudioSubsystem) -> Result<Self, String> {
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        Ok(Self {
            channels: [audio.open_queue(None, &spec)?],
        })
    }
}

impl AudioOutput for Sdl2AudioOutput {
    fn queue(&mut self, channel: usize, samples: &[Sample]) {
        self.channels[channel].queue(samples);
    }

    fn on(&mut self, channel: usize) {
        self.channels[channel].resume();
    }

    fn off(&mut self, channel: usize) {
        self.channels[channel].pause();
    }

    fn clear(&mut self, channel: usize) {
        self.channels[channel].clear();
    }
}
