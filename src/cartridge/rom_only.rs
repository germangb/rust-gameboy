use crate::device::Device;

#[derive(Debug)]
pub struct RomOnly {
    rom: Box<[u8]>,
}

impl RomOnly {
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
