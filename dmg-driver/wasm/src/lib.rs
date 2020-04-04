/// Audio backend using WebAudio.
///
/// The feature *audio* must be enabled.
#[cfg(feature = "audio")]
pub mod apu;
/// Poker Camera emulation using an HTMLVideoElement.
///
/// The feature *poket-camera* must be enabled.
#[cfg(feature = "poket-camera")]
pub mod poket_camera;
/// Video backend using Canvas.
///
/// The feature *video* must be enabled.
#[cfg(feature = "video")]
pub mod ppu;
