# `dmg`

GameBoy emulation in Rust

![](assets/zelda.gif)
![](assets/mario.gif)

## Usage

Example using SDL for video and [Rodio] for audio.

(A simplified version of the full example in [native frontend])

[Rodio]: https://github.com/tomaka/rodio
[native frontend]: dmg-frontend/native/src/main.rs

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
| Color (CGB)    | ✔️       | Still buggy. Working on it
| Sound          |         | Still buggy. Working on it
| Link cable     |         | In scope but not implemented yet.

## Peripherals

These are the supported peripherals in the [dmg-peripheral](dmg-peripheral) module.

| Peripheral | Requirements                | Notes 
| ---        | ---                         | ---
| Camera     | `DMG_PERIPHERAL_CAMERA_ROM` | You must provide your own rom in via the environment variable

Implementing new peripherals is straightforward:

```Rust
use dmg_lib::map::Mapped;
use dmg_lib::cartridge::Cartridge;

struct MyPeripheral { .. }

// peripherals are mapped to:
//
//  0x0000..=0x7fff (ROM area)
//  0xa000..=0xbfff (External RAM area)
//
// (ref: http://problemkaputt.de/pandocs.htm#memorymap):
impl Mapped for MyPeripherak { 
    fn read(&self, addr: u16) -> u8 {
        // ...
    }

    fn write(&mut self, addr: u16, data: u8) {
        // ...
    }
}

// Add the Cartridge marker trait
impl Cartridge for MyPeripheral {}
```

## Boot ROMs

In order to include the bootstrap rom, you must own boot roms for both GB and CGB (they can be found online easily).

```bash
# these must be present when you cargo build your crate
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

```bash
cargo test cpu_instrs
```

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
