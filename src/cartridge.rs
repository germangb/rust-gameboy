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
            0xa000..=0xbfff => 0,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => {}
            0xa000..=0xbfff => {}
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {}
