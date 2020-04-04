use dmg_lib::apu::{device::AudioDevice, samples::SamplesMutex};
use sdl2::{
    audio::{AudioCallback, AudioDevice as SdlAudioDevice, AudioFormatNum, AudioSpecDesired},
    AudioSubsystem,
};

/// Audio callback.
pub struct Callback<D: AudioDevice>(SamplesMutex<D>);

impl<D> AudioCallback for Callback<D>
where
    D: AudioDevice + 'static,
    D::Sample: AudioFormatNum,
{
    type Channel = D::Sample;

    fn callback(&mut self, samples: &mut [Self::Channel]) {
        let lock = self.0.lock();
        for (src, dst) in lock.zip(samples) {
            *dst = src;
        }
    }
}

/// Wraps APU in an SDL audio device.
pub fn create_device<D>(
    audio: &AudioSubsystem,
    samples: SamplesMutex<D>,
) -> Result<SdlAudioDevice<Callback<D>>, String>
where
    D: AudioDevice + 'static,
    D::Sample: AudioFormatNum,
{
    let spec = AudioSpecDesired {
        freq: Some(D::sample_rate() as _),
        channels: Some(if D::mono() { 1 } else { 2 }),
        samples: Some(735),
    };

    audio.open_playback(None, &spec, |spec| Callback(samples))
}
