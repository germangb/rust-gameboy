use crate::device::Device;

pub trait Cartridge: Device {}

pub struct RomOnly {
    rom: Box<[u8]>,
}

impl RomOnly {
    pub fn tetris() -> Self {
        let rom = include_bytes!("../roms/Tetris-USA.gb")
            .to_vec()
            .into_boxed_slice();
        Self { rom }
    }
}

impl Cartridge for RomOnly {}

impl Device for RomOnly {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => *self.rom.get(addr as usize).unwrap_or(&0),
            0xa000..=0xbfff => panic!(),
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => {}
            0xa000..=0xbfff => panic!(),
            _ => panic!(),
        }
    }
}

pub struct RomRam {
    rom: Box<[u8]>,
    ram: Box<[u8; 0x2000]>,
}

impl RomRam {
    pub fn dr_mario() -> Self {
        let rom = include_bytes!("../roms/cpu_instrs/individual/02-interrupts.gb")
            .to_vec()
            .into_boxed_slice();
        Self {
            rom,
            ram: Box::new([0; 0x2000]),
        }
    }
}

impl Cartridge for RomRam {}

impl Device for RomRam {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => *self.rom.get(addr as usize).unwrap_or(&0),
            0xa000..=0xbfff => self.ram[addr as usize - 0xa000],
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => {}
            0xa000..=0xbfff => self.ram[addr as usize - 0xa000] = data,
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {}
