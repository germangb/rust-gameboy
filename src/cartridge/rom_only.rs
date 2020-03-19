use crate::device::Device;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct RomOnly {
    rom: Box<[u8]>,
    #[cfg_attr(feature = "serialize", serde(serialize_with = "crate::serde::ser_boxed_8k"))]
    #[cfg_attr(feature = "serialize", serde(deserialize_with = "crate::serde::de_boxed_8k"))]
    ram: Box<[u8; 0x2000]>,
}

impl RomOnly {
    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        Self {
            rom: rom.into(),
            ram: Box::new([0; 0x2000]),
        }
    }
}

impl Device for RomOnly {
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
