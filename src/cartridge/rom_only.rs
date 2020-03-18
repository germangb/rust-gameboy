use crate::{cartridge::Cartridge, device::Device};

#[derive(Debug)]
pub struct RomOnly {
    rom: Box<[u8]>,
}

impl RomOnly {
    pub fn tetris() -> Self {
        let rom = include_bytes!("../../roms/Tetris-USA.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn print10_demo() -> Self {
        let rom = include_bytes!("../../roms/10-print.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn dr_mario() -> Self {
        let rom = include_bytes!("../../roms/Dr. Mario (World).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn tennis() -> Self {
        let rom = include_bytes!("../../roms/Tennis (JUE) [!].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn test() -> Self {
        let rom =
            include_bytes!("../../roms/gb-test-roms/interrupt_time/interrupt_time.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        Self { rom: rom.into() }
    }
}

impl Device for RomOnly {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => *self.rom.get(addr as usize).unwrap_or(&0),
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, _: u8) {
        match addr {
            0x0000..=0x7fff => {}
            _ => panic!(),
        }
    }
}

impl Cartridge for RomOnly {}
