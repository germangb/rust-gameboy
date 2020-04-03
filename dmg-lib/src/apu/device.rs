use num_traits::{Bounded, Num, NumCast};

/// Audio device
pub trait AudioDevice {
    type Sample: Copy + Bounded + NumCast;

    /// Return the samples per second of the device.
    fn sample_rate() -> u64;

    /// Returns true if the channel is single-channel.
    fn mono() -> bool;
}

/// 44100Hz, stereo, 16bit samples.
pub enum Stereo16i44100 {}
/// 44100Hz, mono, 16bit samples.
pub enum Mono16i44100 {}

impl AudioDevice for Stereo16i44100 {
    type Sample = i16;

    fn sample_rate() -> u64 {
        44100
    }

    fn mono() -> bool {
        false
    }
}

impl AudioDevice for Mono16i44100 {
    type Sample = i16;

    fn sample_rate() -> u64 {
        44100
    }

    fn mono() -> bool {
        true
    }
}

/// A stub device, meant for emulators without sound.
///
/// # Panic
/// Since this device is meant for emulators without sound, calling any method
/// will panic.
impl AudioDevice for () {
    type Sample = i16;

    fn sample_rate() -> u64 {
        panic!()
    }

    fn mono() -> bool {
        panic!()
    }
}
