use crate::{device::Device, interrupts::Interrupts};
use std::{cell::RefCell, rc::Rc};

// Mode 0 is present between 201-207 clks, 2 about 77-83 clks, and 3 about
// 169-175 clks. A complete cycle through these states takes 456 clks. VBlank
// lasts 4560 clks. A complete screen refresh occurs every 70224 clks.)
pub const HBLANK: usize = 204;
pub const OAM: usize = 80;
pub const PIXEL: usize = 172;
pub const VBLANK: usize = 4560;

#[repr(u8)]
#[derive(Clone, Copy)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    Pixel = 3,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Color {
    White = 0xffffff,
    LightGray = 0xaaaaaa,
    DarkGray = 0x555555,
    Black = 0x000000,
}

pub struct Ppu {
    buffer: [Color; 160 * 144],
    cycles: usize,
    vram: [u8; 0x2000],
    oam: [u8; 0x9f],
    mode: Mode,
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    wy: u8,
    wx: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    int: Rc<RefCell<Interrupts>>,
}

impl Ppu {
    pub fn new(int: Rc<RefCell<Interrupts>>) -> Self {
        Self {
            buffer: [Color::White; 160 * 144],
            cycles: 0,
            vram: [0; 0x2000],
            oam: [0; 0x9f],
            mode: Mode::OAM,
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            int,
        }
    }

    pub fn step(&mut self, cycles: usize) {
        unimplemented!()
    }

    fn render_line(&mut self) {
        unimplemented!()
    }

    fn stat(&self) -> u8 {
        (self.stat & 0xfc) | self.mode as u8
    }
}

impl Device for Ppu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.vram[addr as usize - 0x8000],
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00],
            0xff40 => self.lcdc,
            0xff41 => self.stat(),
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff4a => self.wy,
            0xff4b => self.wx,
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9fff => self.vram[addr as usize - 0x8000] = data,
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00] = data,
            0xff40 => self.lcdc = data,
            0xff41 => self.stat = data,
            0xff42 => self.scy = data,
            0xff43 => self.scx = data,
            0xff44 => self.ly = data,
            0xff45 => self.lyc = data,
            0xff4a => self.wy = data,
            0xff4b => self.wx = data,
            0xff47 => self.bgp = data,
            0xff48 => self.obp0 = data,
            0xff49 => self.obp1 = data,
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {}
