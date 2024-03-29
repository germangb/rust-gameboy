#![cfg_attr(
    not(debug_assertions),
    deny(
        dead_code,
        unused_imports,
        unused_variables,
        clippy::style,
        clippy::correctness,
        clippy::complexity,
        clippy::perf,
    )
)]
#![deny(clippy::style, clippy::correctness, clippy::complexity, clippy::perf)]

/// Audio dmg-backend using WebAudio.
///
/// The feature *audio* must be enabled.
#[cfg(feature = "audio")]
pub mod apu;
/// Poker Camera emulation using an HTMLVideoElement.
///
/// The feature *poket-camera* must be enabled.
#[cfg(feature = "poket-camera")]
pub mod poket_camera;
/// Video dmg-backend using Canvas.
///
/// The feature *video* must be enabled.
#[cfg(feature = "video")]
pub mod ppu;
