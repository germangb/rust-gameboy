use crate::dev::Device;

const VRAM_SIZE: usize = 0x2000;

pub struct VideoRam {
    vram: [[u8; VRAM_SIZE]; 2],
    vbk: u8,
}

impl Default for VideoRam {
    fn default() -> Self {
        Self {
            vram: [[0; VRAM_SIZE]; 2],
            vbk: 0,
        }
    }
}

impl VideoRam {
    pub fn bank_0(&self) -> &[u8; 0x2000] {
        &self.vram[0]
    }

    pub fn bank_1(&self) -> &[u8; 0x2000] {
        &self.vram[1]
    }
}

impl Device for VideoRam {
    fn read(&self, addr: u16) -> u8 {
        match addr as usize {
            addr @ 0x8000..=0x9fff => {
                let bank = (self.vbk & 0x1) as usize;
                self.vram[bank][addr - 0x8000]
            }
            0xff4f => self.vbk,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr as usize {
            addr @ 0x8000..=0x9fff => {
                let bank = (self.vbk & 0x1) as usize;
                self.vram[bank][addr - 0x8000] = data;
            }
            0xff4f => self.vbk = data,
            _ => panic!(),
        }
    }
}
