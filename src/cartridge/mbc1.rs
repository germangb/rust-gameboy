use crate::{cartridge::Cartridge, device::Device};

enum Mode {
    Rom,
    Ram,
}

#[allow(dead_code)]
pub struct Mbc1 {
    rom: Box<[u8]>,
    ram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    mode: Mode,
}

impl Mbc1 {
    pub fn test() -> Self {
        let rom = include_bytes!("../../roms/gb-test-roms/cpu_instrs/cpu_instrs.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn test_oam_bug() -> Self {
        let rom =
            include_bytes!("../../roms/gb-test-roms/oam_bug/rom_singles/1-lcd_sync.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn pocket_demo() -> Self {
        let rom = include_bytes!("../../roms/pocket.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn oh_demo() -> Self {
        let rom = include_bytes!("../../roms/oh.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn gejmboj_demo() -> Self {
        let rom = include_bytes!("../../roms/gejmboj.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn jml_a09_demo() -> Self {
        let rom = include_bytes!("../../roms/jml-a09.gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn metroid() -> Self {
        let rom = include_bytes!("../../roms/Metroid II - Return of Samus (UE).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn ducktales() -> Self {
        let rom = include_bytes!("../../roms/Duck Tales (USA).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn ducktales_2() -> Self {
        let rom = include_bytes!("../../roms/Duck Tales 2 (USA).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn contra() -> Self {
        let rom = include_bytes!("../../roms/Contra (J).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn aladdin() -> Self {
        let rom = include_bytes!("../../roms/Aladdin (U) [S][!].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn zelda() -> Self {
        let rom =
            include_bytes!("../../roms/Legend of Zelda, The - Link's Awakening (U) (V1.2) [!].gb")
                .to_vec();
        Self::from_bytes(rom)
    }

    pub fn donkey_kong_land() -> Self {
        let rom = include_bytes!("../../roms/Donkey Kong Land (USA, Europe).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn mario() -> Self {
        let rom = include_bytes!("../../roms/Super Mario Land (World).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn mario_2() -> Self {
        let rom =
            include_bytes!("../../roms/Super Mario Land 2 - 6 Golden Coins (UE) (V1.0) [!].gb")
                .to_vec();
        Self::from_bytes(rom)
    }

    pub fn mario_4() -> Self {
        let rom = include_bytes!("../../roms/Super Mario Land 4 (J) [!].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn batman() -> Self {
        let rom = include_bytes!("../../roms/Batman (JU) [!].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn batman_animated() -> Self {
        let rom = include_bytes!("../../roms/Batman - The Animated Series (U).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn worms() -> Self {
        let rom = include_bytes!("../../roms/Worms (U) [!].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn pinball() -> Self {
        let rom = include_bytes!("../../roms/Pinball Deluxe (U).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn f1_race() -> Self {
        let rom = include_bytes!("../../roms/F-1 Race (JUE) (v1.1).gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn race_drivin() -> Self {
        let rom = include_bytes!("../../roms/Race Drivin' (U) [b2].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn ferrari() -> Self {
        let rom = include_bytes!("../../roms/Ferrari - Grand Prix Challenge (U) [!].gb").to_vec();
        Self::from_bytes(rom)
    }

    pub fn v_rally() -> Self {
        let rom =
            include_bytes!("../../roms/V-Rally - Championship Edition (Europe) (En,Fr,De).gb")
                .to_vec();
        Self::from_bytes(rom)
    }

    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        let ram_banks = 4;
        Self {
            rom: rom.into(),
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
            0x0000..=0x3fff => *self.rom.get(addr as usize).unwrap_or(&0),
            0x4000..=0x7fff => self.rom[0x4000 * self.rom_bank.max(1) + addr as usize - 0x4000],
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

impl Cartridge for Mbc1 {}
