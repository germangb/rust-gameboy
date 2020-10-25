use crate::device::Device;

const SIZE: usize = 0x1000;

// In CGB Mode 32 KBytes internal RAM are available. This memory is divided into
// 8 banks of 4 KBytes each. Bank 0 is always available in memory at C000-CFFF,
// Bank 1-7 can be selected into the address space at D000-DFFF.
// Bit 0-2  Select WRAM Bank (Read/Write)
pub struct WRam {
    svbk: u8,
    wram: Box<[[u8; SIZE]; 8]>,
}

impl WRam {
    /// Return the contents of the SVBK (WRAM bank select) register.
    pub fn svbk(&self) -> u8 {
        self.svbk
    }

    /// Get the contents of the WRAM memory banks.
    ///
    /// # Panic
    /// Panics if `bank` > 7
    pub fn bank(&self, bank: usize) -> &[u8; SIZE] {
        assert!(bank < 8);
        &self.wram[bank]
    }

    /// Get the contents of the WRAM memory banks as mutable.
    ///
    /// # Panic
    /// Panics if `bank` > 7
    pub fn bank_mut(&mut self, bank: usize) -> &mut [u8; SIZE] {
        assert!(bank < 8);
        &mut self.wram[bank]
    }
}

impl Default for WRam {
    fn default() -> Self {
        Self { svbk: 0x1,
               wram: Box::new([[0; SIZE]; 8]) }
    }
}

impl Device for WRam {
    fn read(&self, addr: u16) -> u8 {
        match addr as usize {
            addr @ 0xc000..=0xcfff => self.wram[0][addr - 0xc000],
            addr @ 0xd000..=0xdfff => {
                let bank = (self.svbk & 0x7) as usize;
                self.wram[bank.max(1)][addr - 0xd000]
            }
            addr @ 0xe000..=0xfdff => {
                let addr = (addr as u16) - 0xe000 + 0xc000;
                self.read(addr)
            }
            0xff70 => self.svbk,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr as usize {
            addr @ 0xc000..=0xcfff => self.wram[0][addr - 0xc000] = data,
            addr @ 0xd000..=0xdfff => {
                let bank = (self.svbk & 0x7) as usize;
                self.wram[bank.max(1)][addr - 0xd000] = data;
            }
            addr @ 0xe000..=0xfdff => {
                let addr = (addr as u16) - 0xe000 + 0xc000;
                self.write(addr, data);
            }
            0xff70 => self.svbk = data,
            _ => panic!(),
        }
    }
}
