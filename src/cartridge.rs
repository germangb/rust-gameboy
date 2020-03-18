use crate::device::Device;

mod mbc1;
mod mbc3;
mod rom_only;

pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use rom_only::RomOnly;

pub trait Cartridge: Device {
    /// Steps internal clock (only relevant for MBC3 Roms).
    #[allow(unused_variables)]
    fn step(&mut self, cycles: usize) {}
}
