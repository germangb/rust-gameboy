use crate::device::Device;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct RomOnly {
    rom: Box<[u8]>,
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

#[allow(unused_variables)]
#[cfg(feature = "serialize")]
impl Serialize for RomOnly {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

#[allow(unused_variables)]
#[cfg(feature = "serialize")]
impl<'de> Deserialize<'de> for RomOnly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
