use crate::map::Mapped;

mod mbc1;
mod mbc3;
mod mbc5;
mod rom;

pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use mbc5::Mbc5;
pub use rom::Rom;

// 0149 - RAM Size
// Specifies the size of the external RAM in the cartridge (if any).
// 00h - None
// 01h - 2 KBytes
// 02h - 8 Kbytes
// 03h - 32 KBytes (4 banks of 8KBytes each)
#[allow(unused_variables)]
fn ram_banks(banks: u8) -> usize {
    // match banks {
    //     0x00 => 0,
    //     0x01 | 0x02 => 1,
    //     0x03 => 4,
    //     0x04 => 16,
    //     _ => panic!(),
    // }
    16
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CGBSupport {
    /// Supports both GB & CGB.
    CGB_AND_GB = 0x80,
    /// Supports CGB only.
    CGB_ONLY = 0xc0,
}

pub trait Cartridge: Mapped {
    #[allow(unused_variables)]
    fn step(&mut self, cycles: u64) {}

    /// CGB support flag.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use dmg_lib::cartridge::{Mbc3, Cartridge};
    ///
    /// # let bytes = &[];
    /// let rom = Mbc3::from_bytes(bytes);
    ///
    /// if rom.cgb_support().is_some() {
    ///     println!("CGB mode supported");
    /// } else {
    ///     println!("CGB not supported");
    /// }
    /// ```
    fn cgb_support(&self) -> Option<CGBSupport> {
        match self.read(0x143) {
            0x80 => Some(CGBSupport::CGB_AND_GB),
            0xc0 => Some(CGBSupport::CGB_ONLY),
            _ => None,
        }
    }
}

impl Mapped for () {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x143 => 0xc0,
            _ => 0xff,
        }
    }

    fn write(&mut self, _: u16, _: u8) {}
}

impl Cartridge for () {}
impl Cartridge for Rom {}
impl Cartridge for Mbc1 {}
impl Cartridge for Mbc3 {}
impl Cartridge for Mbc5 {}

impl<C: Cartridge> Cartridge for Box<C> {}
