use std::marker::PhantomData;

pub trait Sample: Copy {
    fn min() -> Self;
    fn max() -> Self;
    fn from_f64(n: f64) -> Self;
    fn as_f64(&self) -> f64;
}

impl Sample for i16 {
    fn min() -> Self {
        std::i16::MIN
    }

    fn max() -> Self {
        std::i16::MAX
    }

    fn from_f64(n: f64) -> Self {
        n as Self
    }

    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

impl Sample for u16 {
    fn min() -> Self {
        std::u16::MIN
    }

    fn max() -> Self {
        std::u16::MAX
    }

    fn from_f64(n: f64) -> Self {
        n as Self
    }

    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

impl Sample for f32 {
    fn min() -> Self {
        -1.0
    }

    fn max() -> Self {
        1.0
    }

    fn from_f64(n: f64) -> Self {
        n as Self
    }

    fn as_f64(&self) -> f64 {
        *self as f64
    }
}

impl Sample for () {
    fn min() -> Self {
        panic!()
    }

    fn max() -> Self {
        panic!()
    }

    fn from_f64(_: f64) -> Self {
        panic!()
    }

    fn as_f64(&self) -> f64 {
        panic!()
    }
}

/// Audio device
pub trait AudioDevice {
    type Sample: Sample;

    /// Return the samples per second of the device.
    fn sample_rate() -> u64;

    /// Returns true if the channel is single-channel.
    fn mono() -> bool;
}

/// 44100Hz, stereo.
pub struct Stereo44100<T>(PhantomData<T>);
/// 44100Hz, mono.
pub struct Mono44100<T>(PhantomData<T>);

impl<T: Sample> AudioDevice for Stereo44100<T> {
    type Sample = T;

    fn sample_rate() -> u64 {
        44100
    }

    fn mono() -> bool {
        false
    }
}

impl<T: Sample> AudioDevice for Mono44100<T> {
    type Sample = T;

    fn sample_rate() -> u64 {
        44100
    }

    fn mono() -> bool {
        false
    }
}

/// A stub device, meant for emulators without sound.
///
/// # Panic
/// Since this device is meant for emulators without sound, calling any method
/// will panic.
impl AudioDevice for () {
    type Sample = ();

    fn sample_rate() -> u64 {
        panic!()
    }

    fn mono() -> bool {
        panic!()
    }
}
