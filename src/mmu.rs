use crate::{
    cartridge::Cartridge, device::Device, interrupts::Interrupts, joypad::Joypad, ppu::Ppu,
    sound::Sound, timer::Timer,
};
use std::{cell::RefCell, rc::Rc};

pub struct Mmu {
    boot: u8,
    cartridge: Box<dyn Cartridge>,
    ppu: Ppu,
    timer: Timer,
    wram: [u8; 0x2000],
    joy: Joypad,
    sound: Sound,
    hram: [u8; 0x7f],
    pub(crate) int: Rc<RefCell<Interrupts>>,
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
            wram: [0; 0x2000],
            joy: Joypad::new(Rc::clone(&int)),
            sound: Sound::new(Rc::clone(&int)),
            hram: [0; 0x7f],
            int,
        }
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

    fn dma(&mut self, d: u8) {
        for addr in 0..=0x9f {
            let src = u16::from(d) << 8 | (addr as u16);
            let dst = 0xfe00 | (addr as u16);
            self.write(dst, self.read(src));
        }
    }

    pub fn step(&mut self, cycles: usize) {
        self.ppu.step(cycles);
        self.timer.step(cycles);
    }

    fn boot(&self) -> bool {
        (self.read(0xff50) & 0x1) != 1
    }
}

impl Device for Mmu {
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
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x000..=0x00ff if self.boot() => include_bytes!("../roms/dmg_boot.bin")[addr as usize],
            0x0000..=0x7fff => self.cartridge.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr),
            0xa000..=0xbfff => self.cartridge.read(addr),
            0xc000..=0xcfff => self.wram[addr as usize - 0xc000],
            0xd000..=0xdfff => self.wram[addr as usize - 0xc000],
            // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
            0xe000..=0xfdff => self.wram[addr as usize - 0xe000],
            0xfe00..=0xfe9f => self.ppu.read(addr),
            #[rustfmt::skip]
            0xfea0..=0xfeff => { /* Not Usable */ 0x0 }
            0xff00..=0xff7f => match addr {
                0xff00 => self.joy.read(addr),
                0xff01 | 0xff02 => {
                    /* serial data transfer (link cable) */
                    0
                }
                0xff04..=0xff07 => self.timer.read(addr),
                0xff0f => self.int.borrow().read(addr),
                0xff10..=0xff14
                | 0xff16..=0xff19
                | 0xff1a..=0xff1e
                | 0xff30..=0xff3f
                | 0xff20..=0xff26 => self.sound.read(addr),
                0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.read(addr),
                0xff46 => 0x0,
                0xff50 => self.boot,
                addr => {
                    eprintln!("unhandled addr = {:x}", addr);
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
            0x000..=0x00ff if self.boot() => { /* read only boot rom */ }
            0x0000..=0x7fff => self.cartridge.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data),
            0xa000..=0xbfff => self.cartridge.write(addr, data),
            0xc000..=0xcfff => self.wram[addr as usize - 0xc000] = data,
            0xd000..=0xdfff => self.wram[addr as usize - 0xc000] = data,
            // E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
            0xe000..=0xfdff => self.wram[addr as usize - 0xe000] = data,
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
                | 0xff20..=0xff26 => self.sound.write(addr, data),
                0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.write(addr, data),
                0xff46 => self.dma(data),
                0xff50 => self.boot = data,
                addr => eprintln!("unhandled addr = {:x}", addr),
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
mod tests {}
