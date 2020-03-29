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
    /// Return the contents of the VBK (VRAM bank select) register.
    pub fn vbk(&self) -> u8 {
        self.vbk
    }

    /// Return the currently mapped bank (according to the VBK register).
    pub fn active(&self) -> &[u8; VRAM_SIZE] {
        let bank = self.vbk & 0x1;
        self.bank(bank as usize)
    }

    /// Return the currently mapped bank (according to the VBK register) as
    /// mutable.
    pub fn active_mut(&mut self) -> &mut [u8; VRAM_SIZE] {
        let bank = self.vbk & 0x1;
        self.bank_mut(bank as usize)
    }

    /// Get the contents of the VRAM memory banks.
    ///
    /// # Panic
    /// Panics if `bank` > 1
    pub fn bank(&self, bank: usize) -> &[u8; VRAM_SIZE] {
        assert!(bank < 2);
        &self.vram[bank]
    }

    /// Get the contents of the VRAM memory banks as mutable.
    ///
    /// # Panic
    /// Panics if `bank` > 1
    pub fn bank_mut(&mut self, bank: usize) -> &mut [u8; VRAM_SIZE] {
        assert!(bank < 2);
        &mut self.vram[bank]
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
