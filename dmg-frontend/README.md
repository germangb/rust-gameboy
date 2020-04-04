# `dmg-frontend`

Reference frontends:

- `native` native frontend using SDL for video and Rodio for audio playback.
- `web` web frontend for the camera peripheral.

## Example frontend

Example using SDL for video and [Rodio] for audio.

(This is a simplified version of the full example in [native frontend])

[Rodio]: https://github.com/tomaka/rodio
[native frontend]: native/src/main.rs

```rust
use dmg_lib::{
    joypad::{Key, Btn},
    apu::device::Stereo44100,
    cartridge::Mbc5,
};
use dmg_driver_rodio::apu::RodioSamples;
use dmg_driver_sdl::ppu::SdlVideoOutput;

// create SDL canvas
let canvas = ...;

// create cartridge
// (this is any type that implements the Cartridge trait)
let cartridge = Mbc5::new(..);

// set up the emulator
let mut dmg = Builder::default()
    .with_cartridge(cartridge)
    .with_video(Sdl2VideoOutput::from_canvas(canvas))
    .with_audio::<Stereo44100<i16>>()
    .build();

// set up audio
let device = rodio::default_output_device().unwrap();
let queue = rodio::Sink::new(&device);
queue.append(RodioSamples::new(dmg.mmu().apu().samples()));
queue.play();

loop {
    // Here you would handle input
    // As an example, press and release the start button
    dmg.mmu_mut().joypad_mut().press(Key::Btn(Btn::Start))
    dmg.mmu_mut().joypad_mut().release(Key::Btn(Btn::Start))
    
    // Emulate one frame and present the result
    dmg.emulate_frame();
    dmg.mmu_mut().ppu_mut().video_mut().present()

    sync(60);
}
```

