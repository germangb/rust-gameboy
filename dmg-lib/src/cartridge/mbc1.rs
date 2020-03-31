use crate::{cartridge::ram_banks, dev::Device};

enum Mode {
    Rom,
    Ram,
}

/// MBC1 controller.
#[rustfmt::skip]
pub struct Mbc1 {
    rom: Box<[u8]>,
    ram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    mode: Mode,
}

impl Mbc1 {
    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        let rom = rom.into();
        let ram_banks = ram_banks(rom[0x149]);
        Self {
            rom,
            ram: vec![[0; 0x2000]; ram_banks],
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            mode: Mode::Rom,
        }
    }
}

impl Device for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => *self.rom.get(addr as usize).unwrap_or(&0xff),
            0x4000..=0x7fff => {
                let rom_bank = self.rom_bank.max(1);
                *self
                    .rom
                    .get(0x4000 * rom_bank + addr as usize - 0x4000)
                    .unwrap_or(&0)
            }
            0xa000..=0xbfff => {
                if self.ram_enable {
                    self.ram[self.ram_bank][addr as usize - 0xa000]
                } else {
                    0
                }
            }
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // Before external RAM can be read or written, it must be enabled by writing to this
            // address space. It is recommended to disable external RAM after accessing it, in order
            // to protect its contents from damage during power down of the gameboy. Usually the
            // following values are used:
            0x0000..=0x1fff => self.ram_enable = data & 0xf == 0xa,
            // Writing to this address space selects the lower 5 bits of the ROM Bank Number (in
            // range 01-1Fh). When 00h is written, the MBC translates that to bank 01h also. That
            // doesn't harm so far, because ROM Bank 00h can be always directly accessed by reading
            // from 0000-3FFF.
            0x2000..=0x3fff => {
                self.rom_bank &= 0x60;
                self.rom_bank |= data as usize & 0x1f;
            }
            // This 2bit register can be used to select a RAM Bank in range from 00-03h, or to
            // specify the upper two bits (Bit 5-6) of the ROM Bank number, depending on the current
            // ROM/RAM Mode. (See below.)
            0x4000..=0x5fff => match self.mode {
                Mode::Rom => {
                    self.rom_bank &= 0x1f;
                    self.rom_bank |= (data as usize & 0x3) << 5;
                }
                Mode::Ram => self.ram_bank = data as usize & 0x3,
            },
            0x6000..=0x7fff => {
                self.mode = match data {
                    0x00 => Mode::Rom,
                    0x01 => Mode::Ram,
                    _ => panic!(),
                }
            }
            0xa000..=0xbfff => self.ram[self.ram_bank][addr as usize - 0xa000] = data,
            addr => panic!("{:x}", addr),
        }
    }
}
