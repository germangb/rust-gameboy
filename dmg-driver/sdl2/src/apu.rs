use dmg_lib::apu::AudioOutput;
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

pub struct Sdl2AudioOutput {
    queue: AudioQueue<i16>,
}

impl Sdl2AudioOutput {
    pub fn new(audio: &AudioSubsystem) -> Result<Self, String> {
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let queue = audio.open_queue(None, &spec)?;
        queue.resume();

        Ok(Self { queue })
    }
}

impl AudioOutput for Sdl2AudioOutput {
    fn queue(&mut self, samples: &[i16]) {
        let buf = samples.to_vec();
        self.queue.queue(buf.as_slice());
    }
}
