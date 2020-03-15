use crate::{
    device::Device,
    interrupts::{Flag, Interrupts},
};
use std::{cell::RefCell, mem, rc::Rc};

// Mode 0 is present between 201-207 clks, 2 about 77-83 clks, and 3 about
// 169-175 clks. A complete cycle through these states takes 456 clks. VBlank
// lasts 4560 clks. A complete screen refresh occurs every 70224 clks.)
pub const HBLANK: usize = 207 * 4;
pub const OAM: usize = 83 * 4;
pub const PIXEL: usize = 175 * 4;
pub const VBLANK: usize = 4560 * 4;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    Pixel = 3,
}

#[repr(u8)]
enum TileMap {
    X9c00 = 0x8,
    X9800 = 0,
}
#[repr(u8)]
enum TileData {
    X8000 = 0x10,
    X8800 = 0,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Color {
    White = 0xff_ffff,
    LightGray = 0xaa_aaaa,
    DarkGray = 0x55_5555,
    Black = 0x00_0000,
}

pub struct Ppu {
    buffer: [Color; 160 * 144],
    cycles: usize,
    vram: [u8; 0x2000],
    oam: [u8; 0xa0],
    mode: Mode,
    // Bit 7 - LCD Display Enable             (0=Off, 1=On)
    // Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    // Bit 5 - Window Display Enable          (0=Off, 1=On)
    // Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    // Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    // Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    // Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    // Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
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
            buffer: [Color::Black; 160 * 144],
            cycles: 0,
            vram: [0; 0x2000],
            oam: [0; 0xa0],
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
        if self.lcdc & 0x80 == 0 {
            return;
        }
        self.cycles += cycles;
        match self.mode {
            Mode::OAM => {
                if self.cycles >= OAM {
                    self.mode = Mode::Pixel;
                    self.cycles %= OAM;
                }
            }
            Mode::Pixel => {
                if self.cycles >= PIXEL {
                    self.mode = Mode::HBlank;
                    self.cycles %= PIXEL;
                    self.render_bg();
                    self.render_sprites();
                    if self.stat & 0x8 != 0 {
                        self.int.borrow_mut().set(Flag::LCDStat);
                    }
                }
            }
            Mode::HBlank => {
                if self.cycles >= HBLANK {
                    self.cycles %= HBLANK;
                    self.ly += 1;
                    if self.ly == 144 {
                        self.mode = Mode::VBlank;
                        self.int.borrow_mut().set(Flag::VBlank);
                    } else {
                        self.mode = Mode::OAM;
                        if self.stat & 0x20 != 0 {
                            self.int.borrow_mut().set(Flag::LCDStat);
                        }
                    }
                }
            }
            Mode::VBlank => {
                if self.cycles >= VBLANK {
                    self.mode = Mode::OAM;
                    self.cycles %= VBLANK;
                    self.ly = 0;
                } else {
                    let line_vb = self.cycles / (OAM + PIXEL + HBLANK);
                    self.ly = 144 + line_vb as u8;
                }
            }
        }
    }

    pub fn buffer(&self) -> &[Color; 160 * 144] {
        &self.buffer
    }

    fn render_bg(&mut self) {
        let bgp = self.bgp;
        let bg_display = self.lcdc & 0x1 != 0;
        let bg_tile_map = if self.lcdc & 0x8 == 0x8 {
            TileMap::X9c00
        } else {
            TileMap::X9800
        };
        let bg_win_tile_data = if self.lcdc & 0x10 == 0x10 {
            TileData::X8000
        } else {
            TileData::X8800
        };
        let pal = [
            Color::White,
            Color::LightGray,
            Color::DarkGray,
            Color::Black,
        ];
        for pix in 0..160 {
            let y = self.scy.wrapping_add(self.ly) as u16;
            let x = self.scx.wrapping_add(pix as u8) as u16;
            if bg_display {
                let tile_map_idx = 32u16 * (y / 8) + (x / 8);
                let tile = match bg_tile_map {
                    TileMap::X9c00 => self.read(0x9c00 + tile_map_idx),
                    TileMap::X9800 => self.read(0x9800 + tile_map_idx),
                };
                let mut tile_data = [0u8; 16];
                match bg_win_tile_data {
                    TileData::X8000 => {
                        self.read_slice(0x8000 + 16 * u16::from(tile), &mut tile_data[..])
                    }
                    TileData::X8800 => {
                        let tile: i8 = unsafe { mem::transmute(tile) };
                        let tile = (tile as i16 + 128) as u16;
                        self.read_slice(0x8800 + 16 * tile, &mut tile_data[..])
                    }
                }
                let column = 7 - (x & 0x7) as u8;
                let line = y & 0x7;
                let lo = tile_data[line as usize * 2] >> column & 0x1;
                let hi = tile_data[line as usize * 2 + 1] >> column & 0x1;
                let pal_idx = (hi << 1) | lo;
                let col_idx = (bgp >> (2 * pal_idx)) & 0x3;
                self.buffer[160 * self.ly as usize + pix] = pal[col_idx as usize];
            }
        }
    }

    fn render_sprites(&mut self) {
        let obj_display = self.lcdc & 0x2 != 0;
        let pal = [
            Color::White,
            Color::LightGray,
            Color::DarkGray,
            Color::Black,
        ];
        if obj_display {
            for oam in (&self.oam[..]).chunks(4) {
                let y = oam[0] as i16;
                let x = oam[1] as i16;
                let ly = self.ly as i16;
                if ly < y - 15 || ly > y - 7 {
                    // TODO 16 pixel sprites
                    continue;
                }
                let tile = oam[2];
                let flag = oam[3];
                let x_flip = flag & 0x20 != 0;
                let y_flip = flag & 0x40 != 0;
                let spr_pal = if flag & 0x10 != 0 {
                    self.obp1
                } else {
                    self.obp0
                };
                let mut tile_data = [0; 16];
                self.read_slice(0x8000 + 16 * u16::from(tile), &mut tile_data[..]);
                for sy in y - 16..(y - 8) {
                    for sx in x - 8..x {
                        if sx >= 0 && sx < 160 && sy >= 0 && sy < 144 {
                            let mut row = (sy - (y - 16)) as u8;
                            let mut col = 7 - (sx - (x - 8)) as u8;
                            if x_flip {
                                col = 7 - col;
                            }
                            if y_flip {
                                row = 7 - row;
                            }
                            assert!(row <= 7);
                            assert!(col <= 7);
                            let lo = tile_data[row as usize * 2] >> col & 0x1;
                            let hi = tile_data[row as usize * 2 + 1] >> col & 0x1;
                            let pal_idx = (hi << 1) | lo;
                            let col_idx = (spr_pal >> (2 * pal_idx)) & 0x3;
                            let p = (160 * sy + sx) as usize;
                            match pal[col_idx as usize] {
                                Color::White => {}
                                c => self.buffer[p] = c,
                            }
                        }
                    }
                }
            }
        }
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
            0xff40 => {
                self.lcdc = data;
                if self.lcdc & 0x1 == 0 {
                    self.mode = Mode::OAM;
                    self.ly = 0;
                    mem::replace(&mut self.buffer, [Color::White; 160 * 144]);
                }
            }
            0xff41 => self.stat = data,
            0xff42 => self.scy = data,
            0xff43 => self.scx = data,
            0xff44 => {
                // The LY indicates the vertical line to which the present data
                // is transferred to the LCD Driver. The LY can take on any
                // value between 0 through 153. The values between 144 and 153
                // indicate the V-Blank period. Writing will reset the counter.
                self.ly = 0;
            }
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
