# `dmg-peripheral`

Peripherals are cartridges that implement some special functionality.

## Supported

| Peripheral | Requirements                | Notes 
| ---        | ---                         | ---
| Camera     | `DMG_PERIPHERAL_CAMERA_ROM` | You must provide your own rom in via the environment variable

## Implement new peripherals

Implementing new peripherals is straightforward.

The only requirement is that the type is marked with the `Cartridge`.

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

