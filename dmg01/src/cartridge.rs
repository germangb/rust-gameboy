//! Cartridge types.
//!
//! Only the most common cartridge types are implemented. Less common cartridges
//! (such as the camera) are implemented in external crates.
use crate::map::Mapped;

mod mbc1;
mod mbc3;
mod mbc5;
mod rom;

pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use mbc5::Mbc5;
pub use rom::Rom;

/// Bank controller trait.
pub trait Controller: Mapped {}

impl Controller for () {}
impl Controller for Rom {}
impl Controller for Mbc1 {}
impl Controller for Mbc3 {}
impl Controller for Mbc5 {}
impl Controller for Box<dyn Controller> {}

impl Mapped for Box<dyn Controller> {
    fn read(&self, addr: u16) -> u8 {
        self.as_ref().read(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.as_mut().write(addr, data)
    }
}

pub fn from_bytes<B: Into<Box<[u8]>>>(bytes: B) -> Result<Box<dyn Controller>, ()> {
    let bytes = bytes.into();
    match *bytes.get(0x147).ok_or(())? {
        0x00 => Ok(Box::new(Rom::new(bytes))),
        0x01..=0x03 => Ok(Box::new(Mbc1::new(bytes))),
        0x0f..=0x13 => Ok(Box::new(Mbc3::new(bytes))),
        0x19..=0x1e => Ok(Box::new(Mbc5::new(bytes))),
        _ => Err(()),
    }
}

fn ram_banks(banks: u8) -> usize {
    match banks {
        0x00 => 0,
        0x01 | 0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        _ => panic!(),
    }
}
