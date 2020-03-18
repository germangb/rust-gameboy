use crate::device::Device;

mod mbc1;
mod mbc3;
mod rom_only;

pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use rom_only::RomOnly;

// 00h -  32KByte (no ROM banking)
// 01h -  64KByte (4 banks)
// 02h - 128KByte (8 banks)
// 03h - 256KByte (16 banks)
// 04h - 512KByte (32 banks)
// 05h -   1MByte (64 banks)  - only 63 banks used by MBC1
// 06h -   2MByte (128 banks) - only 125 banks used by MBC1
// 07h -   4MByte (256 banks)
// 52h - 1.1MByte (72 banks)
// 53h - 1.2MByte (80 banks)
// 54h - 1.5MByte (96 banks)
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RomSize {
    X32K = 0x0,
    X64K = 0x1,
    X128K = 0x2,
    X256K = 0x3,
    X512K = 0x4,
    X1M = 0x5,
    X2M = 0x6,
    X4M = 0x7,
    X1M128K = 0x52,
    X1M256K = 0x53,
    X1M512K = 0x54,
}

impl RomSize {
    pub fn banks(&self) -> usize {
        match *self as usize {
            idx @ 0x0..=0x7 => [0, 4, 8, 16, 32, 64, 128, 256][idx],
            idx @ 0x52..=0x54 => [72, 80, 95][idx - 0x52],
            _ => unreachable!(),
        }
    }
}

// 00h - None
// 01h - 2 KBytes
// 02h - 8 Kbytes
// 03h - 32 KBytes (4 banks of 8KBytes each)
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RamSize {
    X0 = 0x0,
    X2K = 0x1,
    X8K = 0x2,
    X32K = 0x3,
}

impl RamSize {
    pub fn banks(&self) -> usize {
        [0, 0, 0, 4][*self as usize]
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CGB {
    /// Supports both GB & CGB.
    X80 = 0x80,
    /// Supports CGB only.
    XC0 = 0xc0,
}

pub trait Cartridge: Device {
    #[allow(unused_variables)]
    fn step(&mut self, cycles: usize) {}

    /// Return the ROM size.
    fn rom_size(&self) -> RomSize {
        use RomSize::*;
        match self.read(0x148) as usize {
            idx @ 0x00..=0x07 => [X32K, X64K, X128K, X256K, X512K, X1M, X2M, X4M][idx],
            idx @ 0x52..=0x54 => [X1M128K, X1M256K, X1M512K][idx - 0x52],
            _ => panic!(),
        }
    }

    /// Return the RAM size.
    fn ram_size(&self) -> RamSize {
        use RamSize::*;
        match self.read(0x149) as usize {
            idx @ 0x00..=0x03 => [X0, X2K, X8K, X32K][idx],
            _ => panic!(),
        }
    }

    fn cgb(&self) -> Option<CGB> {
        match self.read(0x143) {
            0x80 => Some(CGB::X80),
            0xc0 => Some(CGB::XC0),
            _ => None,
        }
    }

    fn sgb(&self) -> bool {
        self.read(0x146) == 0x3
    }
}

impl Cartridge for RomOnly {}
impl Cartridge for Mbc1 {}
impl Cartridge for Mbc3 {}
impl<C: Cartridge> Cartridge for Box<C> {}
