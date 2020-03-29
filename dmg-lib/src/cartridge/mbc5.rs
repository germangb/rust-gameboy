use crate::{cartridge::ram_banks, dev::Device};

/// MBC5 controller.
pub struct Mbc5 {
    rom: Box<[u8]>,
    ram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
}

impl Mbc5 {
    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        let rom = rom.into();
        let ram_banks = ram_banks(rom[0x149]);
        Self {
            rom,
            ram: vec![[0; 0x2000]; ram_banks],
            rom_bank: 0,
            ram_bank: 0,
            ram_enabled: true,
        }
    }
}

impl Device for Mbc5 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.rom[0x4000 * self.rom_bank + addr as usize - 0x4000],
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    self.ram[self.ram_bank][addr as usize - 0xa000]
                } else {
                    #[cfg(feature = "logging")]
                    log::warn!(target: "mbc5", "RAM READ is disabled");
                    0
                }
            }
            addr => {
                #[cfg(feature = "logging")]
                log::error!(target: "mbc5", "READ from undefined location {:#04x}", addr);
                panic!()
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // Mostly the same as for MBC1, a value of 0Ah will enable reading and writing to
            // external RAM. A value of 00h will disable it.
            0x0000..=0x1fff => {
                self.ram_enabled = data & 0xf == 0xa;

                #[cfg(feature = "logging")]
                log::info!(target: "mbc5", "RAM enabled = {:#02x} ({})", data, self.ram_enabled);
            }
            // The lower 8 bits of the 9-bit rom bank select is written to the 2000-2FFF area while
            // the upper bit is written to the least significant bit of the 3000-3FFF area.
            0x2000..=0x2fff => {
                self.rom_bank &= !0xff;
                self.rom_bank |= data as usize;

                #[cfg(feature = "logging")]
                log::info!(target: "mbc5", "ROM bank LO = {:x}, bank = ({:x})", data, self.rom_bank);
            }
            0x3000..=0x3fff => {
                self.rom_bank &= 0xff;
                self.rom_bank |= (data as usize & 0x1) << 8;

                #[cfg(feature = "logging")]
                log::info!(target: "mbc5", "ROM bank HI = {:x}, bank = {:x}", data, self.rom_bank);
            }
            // writing a value (XXXXBBBB - X = Don't care, B = bank select bits) into 4000-5FFF area
            // will select an appropriate RAM bank at A000-BFFF if the cart contains RAM. Ram sizes
            // are 64kbit,256kbit, & 1mbit.
            0x4000..=0x5fff => {
                self.ram_bank = (data & 0xf) as usize;

                #[cfg(feature = "logging")]
                log::info!(target: "mbc5", "RAM bank = {:#02x}", self.ram_bank);
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    self.ram[self.ram_bank][addr as usize - 0xa000] = data;
                } else {
                    #[cfg(feature = "logging")]
                    log::warn!(target: "mbc5", "RAM WRITE is disabled");
                }
            }
            addr => {
                #[cfg(feature = "logging")]
                log::error!(target: "mbc5", "WRITE to undefined location {:#04x}", addr);
                panic!()
            }
        }
    }
}
