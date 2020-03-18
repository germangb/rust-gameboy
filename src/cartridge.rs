use crate::device::Device;

mod mbc1;
mod mbc3;
mod rom_only;

pub use mbc1::Mbc1;
pub use mbc3::Mbc3;
pub use rom_only::RomOnly;

pub fn from_bytes<B: Into<Box<[u8]>>>(bytes: B) -> Result<Cartridge, ()> {
    let bytes = bytes.into();
    match bytes.get(0x147) {
        Some(byte) => match byte {
            0x00 | 0x08 | 0x09 => Ok(Cartridge::RomOnly(RomOnly::from_bytes(bytes))),
            0x01..=0x03 => Ok(Cartridge::Mbc1(Mbc1::from_bytes(bytes))),
            0x05 | 0x06 => unimplemented!("MBC2"),
            0x0b..=0x0d => unimplemented!("MMM01"),
            0x0f..=0x13 => Ok(Cartridge::Mbc3(Mbc3::from_bytes(bytes))),
            0x15..=0x17 => unimplemented!("MBC4"),
            //0x19..=0x1e => unimplemented!("MBC5"),
            0x19..=0x1e => Ok(Cartridge::Mbc3(Mbc3::from_bytes(bytes))),
            0xfc => unimplemented!("POKET CAMERA"),
            0xfd => unimplemented!("BANDAI TAMA5"),
            0xfe => unimplemented!("HuC3"),
            0xff => unimplemented!("HuC1"),
            c => panic!("Unrecognized cartridge type {:x}", c),
        },
        None => Err(()),
    }
}

pub enum Cartridge {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
    Mbc3(Mbc3),
}

impl Cartridge {
    #[allow(unused_variables)]
    pub fn step(&mut self, cycles: usize) {
        if let Cartridge::Mbc3(rom) = self {
            // TODO step clock
        }
    }
}

macro_rules! cartridges {
    ($($cart:ident),*) => {$(
        impl From<$cart> for Cartridge {
            fn from(c: $cart) -> Self {
                Cartridge::$cart(c)
            }
        }
    )*}
}

cartridges! {
    RomOnly,
    Mbc1,
    Mbc3
}

impl Device for Cartridge {
    fn read(&self, addr: u16) -> u8 {
        match self {
            Cartridge::RomOnly(rom) => rom.read(addr),
            Cartridge::Mbc1(rom) => rom.read(addr),
            Cartridge::Mbc3(rom) => rom.read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match self {
            Cartridge::RomOnly(rom) => rom.write(addr, data),
            Cartridge::Mbc1(rom) => rom.write(addr, data),
            Cartridge::Mbc3(rom) => rom.write(addr, data),
        }
    }
}
