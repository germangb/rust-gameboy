use crate::cartridge::Cartridge;

static ROM: &[u8] = include_bytes!("../tests/roms/10-print.gb");

pub fn rom() -> impl Cartridge {
    crate::cartridge::RomOnly::from_bytes(ROM)
}
