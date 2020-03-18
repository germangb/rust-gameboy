use crate::cartridge::Cartridge;

static ROM: &[u8] = include_bytes!("../tests/roms/10-print.gb");

pub fn rom() -> Cartridge {
    crate::cartridge::from_bytes(ROM).unwrap()
}
