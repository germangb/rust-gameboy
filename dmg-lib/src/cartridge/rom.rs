use crate::dev::Device;

/// Non-switchable ROM & ram banks
pub struct Rom {
    rom: Box<[u8]>,
    ram: Box<[u8; 0x2000]>,
}

impl Rom {
    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        Self {
            rom: rom.into(),
            ram: Box::new([0; 0x2000]),
        }
    }
}

impl Device for Rom {
    fn read(&self, addr: u16) -> u8 {
        match addr as usize {
            addr @ 0x0000..=0x7fff => *self.rom.get(addr).unwrap_or(&0),
            addr @ 0xa000..=0xbfff => self.ram[addr - 0xa000],
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr as usize {
            0x0000..=0x7fff => {}
            addr @ 0xa000..=0xbfff => self.ram[addr - 0xa000] = data,
            _ => panic!(),
        }
    }
}
