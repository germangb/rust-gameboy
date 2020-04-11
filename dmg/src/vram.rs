use crate::map::Mapped;

const SIZE: usize = 0x2000;

pub struct VRam {
    vram: Box<[[u8; SIZE]; 2]>,
    vbk: u8,
}

impl Default for VRam {
    fn default() -> Self {
        Self {
            vram: Box::new([[0; SIZE]; 2]),
            vbk: 0,
        }
    }
}

impl VRam {
    /// Return the contents of the VBK (VRAM bank select) register.
    pub fn vbk(&self) -> u8 {
        self.vbk
    }

    /// Get the contents of the VRAM memory banks.
    ///
    /// # Panic
    /// Panics if `bank` > 1
    pub fn bank(&self, bank: usize) -> &[u8; SIZE] {
        assert!(bank < 2);
        &self.vram[bank]
    }

    /// Get the contents of the VRAM memory banks as mutable.
    ///
    /// # Panic
    /// Panics if `bank` > 1
    pub fn bank_mut(&mut self, bank: usize) -> &mut [u8; SIZE] {
        assert!(bank < 2);
        &mut self.vram[bank]
    }
}

impl Mapped for VRam {
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
