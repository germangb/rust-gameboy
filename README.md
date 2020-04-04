# `dmg`

GameBoy emulation in Rust

![](assets/zelda.gif)
![](assets/mario.gif)

## Usage

Example usage using SDL for video and [Rodio] for audio.

```rust
use dmg_lib::joypad::{Key, Btn};
use dmg_driver_rodio::apu::RodioSamples;
use dmg_driver_sdl::ppu::SdlVideoOutput;

// create SDL canvas
let canvas = ...;

// setup the emulator
let mut dmg = Builder::default()
    .with_cartridge(..)
    .with_video(Sdl2VideoOutput::from_canvas(canvas))
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

## Modules

- [dmg-lib](dmg-lib) core emulation library (cpu, ppu, apu, cartridges, etc..)
- [dmg-driver](dmg-driver) video & audio backends
- [dmg-peripherals](dmg-peripheral) supported peripherals
- [dmg-frontend](dmg-frontend) reference frontends
    - [native](dmg-frontend/native)
    - [web](dmg-frontend/web)

    
## Features

| Feature        | Support | Notes
| -------------- | :-----: | ---
| Cycle accuracy | ❌      | Out of scope (might change my mind later)
| Classic GB     | ✔️       | Works on most games, except the ones that require cycle accuracy.
| Color GB (CGB) | ✔️       | Still buggy. Working on it
| Sound          |         | Still buggy. Working on it
| Serial         |         | In scope but not implemented yet.

## Peripherals

| Peripheral | Requirements                | Notes 
| ---        | ---                         | ---
| Camera     | `DMG_PERIPHERAL_CAMERA_ROM` | You must provide your own rom in via the environment variable


## Boot ROMs

In order to include the bootstrap rom, you must own boot roms for both GB and CGB (they can be found online easily).

```bash
export DMG_BOOT_GB_ROM="<path_to_gb_boot_rom>"
export DMG_BOOT_CGB_ROM="<path_to_cgb_boot_rom>"
```

Then, in your Cargo.toml you must enable the `boot` feature flag:

```toml
# Cargo.toml

[dependencies.dmg-lib]
features = ["boot"]
```

> **NOTE:** There is currently no build-time validation of the boot rom. Therefore it's not guaranteed that, if the boot roms aren't correct, the build will not work.

## Tests

### CPU instruction tests

| Test                       | Pass
| -------------------------- | :---:
| `01-special.gb`            | ✔️
| `02-interrupts.gb`         | ✔️
| `03-op sp,hl.gb`           | ✔️
| `04-op r,imm.gb`           | ✔️
| `05-op rp.gb`              | ✔️
| `06-ld r,r.gb`             | ✔️
| `07-jr,jp,call,ret,rst.gb` | ✔️
| `08-misc instrs.gb`        | ✔️
| `09-op r,r.gb`             | ✔️
| `10-bit ops.gb`            | ✔️
| `11-op a,(hl).gb`          | ✔️

### CPU timming tests

| Test | Pass
| ---- | :---:

## Tested Games

| Rom | Works | Comments
| --- | ----- | ---

## License

`TODO`

## Resources

- http://problemkaputt.de/pandocs.htm
- https://gbdev.gg8.se/wiki/
- https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
- https://gekkio.fi/files/gb-docs/gbctr.pdf
- https://github.com/gbdev/awesome-gbdev
