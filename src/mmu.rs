use crate::{
    apu::Apu, cartridge::Cartridge, dev::Device, interrupts::Interrupts, joypad::Joypad, ppu::Ppu,
    timer::Timer, wram::WorkRam,
};
use std::{cell::RefCell, rc::Rc};

// 0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
// 4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
// 8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
// A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
// C000-CFFF   4KB Work RAM Bank 0 (WRAM)
// D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
// E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
// FE00-FE9F   Sprite Attribute Table (OAM)
// FEA0-FEFF   Not Usable
// FF00-FF7F   I/O Ports
// FF80-FFFE   High RAM (HRAM)
// FFFF        Interrupt Enable Register
pub struct Mmu {
    boot: u8,
    cartridge: Box<dyn Cartridge>,
    ppu: Ppu,
    timer: Timer,
    wram: WorkRam,
    joy: Joypad,
    apu: Apu,
    hram: [u8; 0x7f],
    int: Rc<RefCell<Interrupts>>,
}

impl Mmu {
    pub fn new<C>(cartridge: C) -> Self
    where
        C: Cartridge + 'static,
    {
        let int = Rc::new(RefCell::new(Interrupts::default()));
        Self {
            boot: 0x0,
            cartridge: Box::new(cartridge),
            ppu: Ppu::new(Rc::clone(&int)),
            timer: Timer::new(Rc::clone(&int)),
            wram: WorkRam::new(),
            joy: Joypad::new(Rc::clone(&int)),
            apu: Apu::new(Rc::clone(&int)),
            hram: [0; 0x7f],
            int,
        }
    }

    pub fn cartridge(&self) -> &dyn Cartridge {
        self.cartridge.as_ref()
    }

    pub fn cartridge_mut(&mut self) -> &mut dyn Cartridge {
        self.cartridge.as_mut()
    }

    pub fn joypad(&self) -> &Joypad {
        &self.joy
    }

    pub fn joypad_mut(&mut self) -> &mut Joypad {
        &mut self.joy
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    pub fn wram(&self) -> &WorkRam {
        &self.wram
    }

    pub fn wram_mut(&mut self) -> &mut WorkRam {
        &mut self.wram
    }

    pub fn apu(&self) -> &Apu {
        &self.apu
    }

    pub fn apu_mut(&mut self) -> &mut Apu {
        &mut self.apu
    }

    pub fn step(&mut self, cycles: usize) {
        self.ppu.step(cycles);
        self.timer.step(cycles);
        self.cartridge.step(cycles);
    }

    fn dma(&mut self, d: u8) {
        for addr in 0..=0x9f {
            let src = u16::from(d) << 8 | (addr as u16);
            let dst = 0xfe00 | (addr as u16);
            self.write(dst, self.read(src));
        }
    }

    fn boot_rom_enabled(&self) -> bool {
        (self.read(0xff50) & 0x1) != 1
    }
}

impl Device for Mmu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x000..=0x00ff if self.boot_rom_enabled() => {
                include_bytes!("../roms/dmg_boot.bin")[addr as usize]
            }
            0x0000..=0x7fff => self.cartridge.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr),
            0xa000..=0xbfff => self.cartridge.read(addr),
            0xc000..=0xdfff => self.wram.read(addr),
            // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
            0xe000..=0xfdff => self.wram.read(addr),
            0xfe00..=0xfe9f => self.ppu.read(addr),
            #[rustfmt::skip]
            0xfea0..=0xfeff => { /* Not Usable */ 0x0 }
            0xff00..=0xff7f => match addr {
                0xff00 => self.joy.read(addr),
                0xff01 | 0xff02 => {
                    // TODO serial data transfer (link cable)
                    0
                }
                0xff04..=0xff07 => self.timer.read(addr),
                0xff0f => self.int.borrow().read(addr),
                0xff10..=0xff14
                | 0xff16..=0xff19
                | 0xff1a..=0xff1e
                | 0xff30..=0xff3f
                | 0xff20..=0xff26 => self.apu.read(addr),
                0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f | 0xff51..=0xff55 | 0xff68..=0xff6b => {
                    self.ppu.read(addr)
                }
                0xff46 => 0x0,
                0xff50 => self.boot,
                0xff70 => self.wram.read(addr),
                _ => {
                    // unhandled address
                    0
                }
            },
            0xff80..=0xfffe => match addr {
                0xff0f => self.int.borrow().read(addr),
                0xff80..=0xfffe => self.hram[addr as usize - 0xff80],
                _ => panic!(),
            },
            0xffff => self.int.borrow().read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x000..=0x00ff if self.boot_rom_enabled() => { /* read only boot rom */ }
            0x0000..=0x7fff => self.cartridge.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data),
            0xa000..=0xbfff => self.cartridge.write(addr, data),
            0xc000..=0xdfff => self.wram.write(addr, data),
            // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
            0xe000..=0xfdff => self.wram.write(addr, data),
            0xfe00..=0xfe9f => self.ppu.write(addr, data),
            #[rustfmt::skip]
            0xfea0..=0xfeff => { /* Not Usable */ }
            0xff00..=0xff7f => match addr {
                0xff00 => self.joy.write(addr, data),
                0xff01 | 0xff02 => { /* serial data transfer (link cable) */ }
                0xff04..=0xff07 => self.timer.write(addr, data),
                0xff0f => self.int.borrow_mut().write(addr, data),
                0xff10..=0xff14
                | 0xff16..=0xff19
                | 0xff1a..=0xff1e
                | 0xff30..=0xff3f
                | 0xff20..=0xff26 => self.apu.write(addr, data),
                0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f | 0xff51..=0xff55 | 0xff68..=0xff6b => {
                    self.ppu.write(addr, data)
                }
                0xff46 => self.dma(data),
                0xff50 => self.boot = data,
                0xff70 => self.wram.write(addr, data),
                _ => {
                    // unhandled address
                }
            },
            0xff80..=0xfffe => match addr {
                0xff0f => self.int.borrow_mut().write(addr, data),
                0xff80..=0xfffe => self.hram[addr as usize - 0xff80] = data,
                _ => panic!(),
            },
            0xffff => self.int.borrow_mut().write(addr, data),
        }
    }
}

#[cfg(tests)]
mod tests {
    use crate::{cartridge::RomAndRam, dev::Device, mmu::Mmu};

    #[test]
    fn dma() {
        let mut mmu = Mmu::new(RomAndRam::tetris());

        mmu.write(0xff50, 1);
        mmu.write(0xff46, 0);

        for addr in 0..=0x9f {
            let rom = mmu.read(addr as u16);
            let oam = mmu.read(0xfe00 | (addr as u16));
            assert_eq!(rom, oam);
        }
    }
}
