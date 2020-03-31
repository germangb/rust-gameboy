use dmg_lib::apu::AudioOutput;
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    AudioSubsystem,
};

pub struct Sdl2AudioOutput {
    frame: u64,
    queue: AudioQueue<i16>,
    buf: Vec<i16>,
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

        println!("{:#?}", queue.spec());

        Ok(Self {
            frame: 0,
            queue,
            buf: Vec::with_capacity(4096),
        })
    }
}

impl AudioOutput for Sdl2AudioOutput {
    fn queue(&mut self, samples: &[i16]) {
        self.frame += 1;
        self.buf.extend_from_slice(samples);

        let frame = self.frame;
        let sec = frame as f64 / 60.0;
        println!("---");
        println!(
            "NEW audio frame {} ({:.02} seconds elapsed)",
            self.frame, sec
        );
        println!(
            "buf size: {} ({:.02} seconds)",
            self.buf.len(),
            self.buf.len() as f64 / 44100.0
        );

        let queued = self.queue.size();
        let sec = queued as f64 / 44100.0;
        println!("queued samples {} ({:.02} seconds)", queued, sec);

        if self.frame == 60 {
            let samples = self.buf.len();
            let sec = self.buf.len() as f64 / 44100.0;
            println!("queued {} samples ({:.02} seconds)", samples, sec);
            self.queue.queue(&self.buf[..]);
            self.buf.clear();
            self.frame = 0;
        }
    }
}
