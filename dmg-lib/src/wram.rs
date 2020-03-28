use crate::dev::Device;

const WRAM_SIZE: usize = 0x1000;

// In CGB Mode 32 KBytes internal RAM are available. This memory is divided into
// 8 banks of 4 KBytes each. Bank 0 is always available in memory at C000-CFFF,
// Bank 1-7 can be selected into the address space at D000-DFFF.
// Bit 0-2  Select WRAM Bank (Read/Write)
pub struct WorkRam {
    svbk: u8,
    wram: [[u8; WRAM_SIZE]; 8],
}

impl Default for WorkRam {
    fn default() -> Self {
        Self {
            svbk: 0x1,
            wram: [[0; WRAM_SIZE]; 8],
        }
    }
}

impl Device for WorkRam {
    fn read(&self, addr: u16) -> u8 {
        match addr as usize {
            addr @ 0xc000..=0xcfff => self.wram[0][addr - 0xc000],
            addr @ 0xd000..=0xdfff => {
                let bank = (self.svbk & 0x7) as usize;
                self.wram[bank.max(1)][addr - 0xd000]
            }
            addr @ 0xe000..=0xfdff => self.read(addr as u16 - 0xe000 + 0xc000),
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
            addr @ 0xe000..=0xfdff => self.write(addr as u16 - 0xe000 + 0xc000, data),
            0xff70 => self.svbk = data,
            _ => panic!(),
        }
    }
}
