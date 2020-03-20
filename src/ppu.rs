use crate::{
    dev::Device,
    interrupts::{Flag, Interrupts},
    ppu::palette::{Color, Palette},
    vram::VideoRam,
};
use std::{cell::RefCell, mem, rc::Rc, slice};

pub mod palette;

// Mode 0 is present between 201-207 clks, 2 about 77-83 clks, and 3 about
// 169-175 clks. A complete cycle through these states takes 456 clks. VBlank
// lasts 4560 clks. A complete screen refresh occurs every 70224 clks.)
pub(crate) const HBLANK: usize = 201;
pub(crate) const OAM: usize = 77;
pub(crate) const PIXEL: usize = 169;
pub(crate) const VBLANK: usize = 4650;

const PIXELS: usize = 160 * 144;

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

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct OamEntry {
    pub ypos: u8,
    pub xpos: u8,
    pub tile: u8,
    pub flag: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Scroll {
    scy: u8,
    scx: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Window {
    wy: u8,
    wx: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Pal {
    bgp: u8,
    obp0: u8,
    obp1: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ColorPal {
    // This register is used to address a byte in the CGBs Background Palette Memory. Each two byte
    // in that memory define a color value. The first 8 bytes define Color 0-3 of Palette 0 (BGP0),
    // and so on for BGP1-7.
    //     Bit 0-5   Index (00-3F)
    //     Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
    // Data can be read/written to/from the specified index address through Register FF69. When the
    // Auto Increment Bit is set then the index is automatically incremented after each <write> to
    // FF69. Auto Increment has no effect when <reading> from FF69, so the index must be manually
    // incremented in that case.
    bgpi: u8,
    obpi: u8,
    bgp: [u8; 0x40],
    obp: [u8; 0x40],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Line {
    ly: u8,
    lyc: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct VRamDma {
    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
    hdma5: u8,
}

pub struct Ppu {
    cycles: usize,
    palette: Palette,
    buffer: [Color; PIXELS],
    back_buffer: [Color; PIXELS],

    vram: VideoRam,
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
    // Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    // Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    // Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    // Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    // Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
    // Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
    stat: u8,
    scroll: Scroll,
    line: Line,
    win: Window,
    vram_dma: VRamDma,
    pal: Pal,
    color_pal: ColorPal,
    int: Rc<RefCell<Interrupts>>,
}

impl Ppu {
    pub fn new(int: Rc<RefCell<Interrupts>>) -> Self {
        let vram_dma = VRamDma {
            hdma1: 0,
            hdma2: 0,
            hdma3: 0,
            hdma4: 0,
            hdma5: 0,
        };
        let scroll = Scroll { scy: 0, scx: 0 };
        let win = Window { wy: 0, wx: 0 };
        let line = Line { ly: 0, lyc: 0 };
        let pal = Pal {
            bgp: 0,
            obp0: 0,
            obp1: 0,
        };
        let palette = palette::GRAYSCALE;
        let color_pal = ColorPal {
            bgpi: 0,
            obpi: 0,
            bgp: [0xff; 0x40],
            obp: [0xff; 0x40],
        };
        Self {
            palette,
            buffer: [palette[0]; PIXELS],
            back_buffer: [palette[0]; PIXELS],
            cycles: 0,
            vram: VideoRam::new(),
            oam: [0; 0xa0],
            mode: Mode::OAM,
            lcdc: 0,
            stat: 0,
            scroll,
            line,
            win,
            vram_dma,
            pal,
            color_pal,
            int,
        }
    }

    pub fn step(&mut self, cycles: usize) {
        if self.lcdc & 0x80 == 0 {
            return;
        }
        self.cycles += cycles;
        //println!("ly={} lyc={} | mode={:?} | stat={:07b} lcdc={:08b}", self.ly,
        // self.lyc, self.mode, self.stat, self.lcdc);

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
                    self.render_line();
                }
            }
            Mode::HBlank => {
                if self.cycles >= HBLANK {
                    self.cycles %= HBLANK;

                    // The gameboy permanently compares the value of the LYC and LY registers. When
                    // both values are identical, the coincident bit in the STAT register becomes
                    // set, and (if enabled) a STAT interrupt is requested.
                    if self.stat & 0x40 != 0 && self.line.ly == self.line.lyc {
                        self.int.borrow_mut().set(Flag::LCDStat);
                    }

                    if self.stat & 0x8 != 0 {
                        self.int.borrow_mut().set(Flag::LCDStat);
                    }

                    self.line.ly += 1;

                    if self.line.ly == 144 {
                        // TODO fix worms rom
                        //let obj_display = self.lcdc & 0x2 != 0;
                        let obj_display = true;
                        if obj_display {
                            self.render_sprites();
                        }

                        self.swap_buffers();

                        self.mode = Mode::VBlank;
                        self.int.borrow_mut().set(Flag::VBlank);

                        if self.stat & 0x10 != 0 {
                            self.int.borrow_mut().set(Flag::LCDStat);
                        }
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
                    self.line.ly = 0;
                    if self.stat & 0x20 != 0 {
                        self.int.borrow_mut().set(Flag::LCDStat);
                    }
                } else {
                    let line_vb = self.cycles / (OAM + PIXEL + HBLANK);
                    self.line.ly = 144 + line_vb as u8;
                }
            }
        }

        if self.line.ly == self.line.lyc {
            self.stat |= 0x4;
        } else {
            self.stat &= !0x4;
        }
    }

    pub fn vram(&self) -> &VideoRam {
        &self.vram
    }

    pub fn vram_mut(&mut self) -> &mut VideoRam {
        &mut self.vram
    }

    pub fn buffer(&self) -> &[Color; 160 * 144] {
        &self.buffer
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn set_palette(&mut self, pal: Palette) {
        self.palette = pal;
    }

    fn bg_tile_map(&self) -> TileMap {
        if self.lcdc & 0x8 == 0x8 {
            TileMap::X9c00
        } else {
            TileMap::X9800
        }
    }

    fn win_tile_map(&self) -> TileMap {
        if self.lcdc & 0x40 != 0 {
            TileMap::X9c00
        } else {
            TileMap::X9800
        }
    }

    fn bg_win_tile_data(&self) -> TileData {
        if self.lcdc & 0x10 != 0 {
            TileData::X8000
        } else {
            TileData::X8800
        }
    }

    fn render_line(&mut self) {
        let bg_display = self.lcdc & 0x1 != 0;
        let window_display = self.lcdc & 0x20 != 0;
        if bg_display {
            self.render_bg();
        }
        if window_display {
            self.render_win();
        }
    }

    fn swap_buffers(&mut self) {
        mem::replace(&mut self.buffer, self.back_buffer);
    }

    fn clear(&mut self) {
        mem::replace(&mut self.buffer, [self.palette[0]; 160 * 144]);
        mem::replace(&mut self.back_buffer, [self.palette[0]; 160 * 144]);
    }

    fn render_win(&mut self) {
        let Window { wy, wx } = self.win;
        let Pal { bgp, .. } = self.pal;
        if self.line.ly < wy || wx >= 160 {
            return;
        }
        let bgp = bgp;
        let win_tile_map = self.win_tile_map();
        let bg_win_tile_data = self.bg_win_tile_data();
        for pix in wx..=166 {
            if pix < 7 {
                continue;
            }
            let y = u16::from(self.line.ly - wy);
            let x = u16::from(pix - wx);
            let pixel = 160 * self.line.ly as usize + (pix - 7) as usize;
            if pixel >= PIXELS {
                continue;
            }

            let tile_map_idx = (32u16 * (y / 8) + (x / 8)) as usize;
            let bank_0 = self.vram.bank_0();
            let bank_1 = self.vram.bank_1();
            let (tile, flags) = match win_tile_map {
                TileMap::X9c00 => (bank_0[0x1c00 + tile_map_idx], bank_1[0x1c00 + tile_map_idx]),
                TileMap::X9800 => (bank_0[0x1800 + tile_map_idx], bank_1[0x1800 + tile_map_idx]),
            };
            let mut col = 7 - (x & 0x7) as u8;
            let mut lin = y & 0x7;
            if flags & 0x20 != 0 {
                col = 7 - col;
            }
            if flags & 0x40 != 0 {
                lin = 7 - lin;
            }

            let (lo, hi) = match bg_win_tile_data {
                TileData::X8000 => {
                    let lo = self.read(0x8000 + 16 * u16::from(tile) + lin * 2) >> col & 0x1;
                    let hi = self.read(0x8000 + 16 * u16::from(tile) + lin * 2 + 1) >> col & 0x1;
                    (lo, hi)
                }
                TileData::X8800 => {
                    let tile: i8 = unsafe { mem::transmute(tile) };
                    let tile = (tile as i16 + 128) as u16;
                    let lo = self.read(0x8800 + 16 * tile + lin * 2) >> col & 0x1;
                    let hi = self.read(0x8800 + 16 * tile + lin * 2 + 1) >> col & 0x1;
                    (lo, hi)
                }
            };
            let pal_idx = (hi << 1) | lo;
            let col_idx = (bgp >> (2 * pal_idx)) & 0x3;
            self.back_buffer[pixel] = self.palette[col_idx as usize];
        }
    }

    fn render_bg(&mut self) {
        let Pal { bgp, .. } = self.pal;
        let bg_tile_map = self.bg_tile_map();
        let bg_win_tile_data = self.bg_win_tile_data();
        let Scroll { scy, scx } = self.scroll;
        for pix in 0..160 {
            let y = scy.wrapping_add(self.line.ly).wrapping_sub(0) as u16;
            let x = (pix as u8).wrapping_add(scx) as u16;

            let tile_map_idx = (32u16 * (y / 8) + (x / 8)) as usize;
            let bank_0 = self.vram.bank_0();
            let bank_1 = self.vram.bank_1();
            let (tile, flags) = match bg_tile_map {
                TileMap::X9c00 => (bank_0[0x1c00 + tile_map_idx], bank_1[0x1c00 + tile_map_idx]),
                TileMap::X9800 => (bank_0[0x1800 + tile_map_idx], bank_1[0x1800 + tile_map_idx]),
            };
            let mut col = 7 - (x & 0x7) as u8;
            let mut lin = y & 0x7;
            if flags & 0x20 != 0 {
                col = 7 - col;
            }
            if flags & 0x40 != 0 {
                lin = 7 - lin;
            }

            let (lo, hi) = match bg_win_tile_data {
                TileData::X8000 => {
                    let lo = self.read(0x8000 + 16 * u16::from(tile) + lin * 2) >> col & 0x1;
                    let hi = self.read(0x8000 + 16 * u16::from(tile) + lin * 2 + 1) >> col & 0x1;
                    (lo, hi)
                }
                TileData::X8800 => {
                    let tile: i8 = unsafe { mem::transmute(tile) };
                    let tile = (tile as i16 + 128) as u16;
                    let lo = self.read(0x8800 + 16 * tile + lin * 2) >> col & 0x1;
                    let hi = self.read(0x8800 + 16 * tile + lin * 2 + 1) >> col & 0x1;
                    (lo, hi)
                }
            };
            let pal_idx = (hi << 1) | lo;
            let col_idx = (bgp >> (2 * pal_idx)) & 0x3;
            self.back_buffer[160 * self.line.ly as usize + pix] = self.palette[col_idx as usize];
        }
    }

    /// Return the list of OAM entries.
    pub fn oam_entries(&self) -> &[OamEntry] {
        unsafe { slice::from_raw_parts(self.oam.as_ptr() as _, 40) }
    }

    /// Return the mutable list of OAM entries.
    pub fn oam_entries_mut(&mut self) -> &mut [OamEntry] {
        unsafe { slice::from_raw_parts_mut(self.oam.as_ptr() as _, 40) }
    }

    fn write_pixel(&mut self, y: usize, x: usize, color: Color) {
        self.back_buffer[160 * y + x] = color;
    }

    fn pixel(&self, y: usize, x: usize) -> Color {
        self.back_buffer[160 * y + x]
    }

    fn write_color_pal(pal: &mut [u8], mut idx: u8, data: u8) -> u8 {
        pal[(idx & 0x3f) as usize] = data;
        if idx & 0x80 != 0 {
            idx += 1;
            idx &= 0xbf;
        }
        idx
    }

    fn render_sprites(&mut self) {
        let Pal { obp0, obp1, .. } = self.pal;
        let mut entries = self.oam_entries().to_vec();
        entries.sort_by_key(|o| o.xpos);
        for oam in entries {
            let tile = u16::from(oam.tile);
            let xpos = i16::from(oam.xpos);
            let ypos = i16::from(oam.ypos);
            let behind_bg = oam.flag & 0x80 != 0;
            let x_flip = oam.flag & 0x20 != 0;
            let y_flip = oam.flag & 0x40 != 0;
            let pal = if oam.flag & 0x10 != 0 { obp1 } else { obp0 };
            let lim = if self.lcdc & 0x4 != 0 { 0 } else { 8 };
            for sy in ypos - 16..ypos - lim {
                for sx in xpos - 8..xpos {
                    if sx >= 0 && sx < 160 && sy >= 0 && sy < 144 {
                        let mut lin = (sy - (ypos - 16)) as u16;
                        let mut col = 7 - (sx - (xpos - 8)) as u8;
                        if x_flip {
                            col = 7 - col
                        }
                        if y_flip {
                            if lim == 8 {
                                lin = 7 - lin;
                            } else {
                                lin = 15 - lin;
                            }
                        }
                        let lo = self.read(0x8000 + 16 * tile + lin * 2) >> col & 0x1;
                        let hi = self.read(0x8000 + 16 * tile + lin * 2 + 1) >> col & 0x1;
                        let pal_idx = (hi << 1) | lo;
                        if pal_idx == 0 {
                            continue;
                        }
                        let sx = sx as usize;
                        let sy = sy as usize;
                        let col_idx = (pal >> (2 * pal_idx)) & 0x3;
                        if behind_bg && self.pixel(sy, sx) != self.palette[0] {
                            continue;
                        }
                        self.write_pixel(sy, sx, self.palette[col_idx as usize]);
                    }
                }
            }
        }
    }
}

impl Device for Ppu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.vram.read(addr),
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00],
            0xff40 => self.lcdc,
            0xff41 => (self.stat & 0xfc) | self.mode as u8,
            0xff42 => self.scroll.scy,
            0xff43 => self.scroll.scx,
            0xff44 => self.line.ly,
            0xff45 => self.line.lyc,
            0xff4a => self.win.wy,
            0xff4b => self.win.wx,
            0xff47 => self.pal.bgp,
            0xff48 => self.pal.obp0,
            0xff49 => self.pal.obp1,
            0xff4f => self.vram.read(addr),
            0xff51 => self.vram_dma.hdma1,
            0xff52 => self.vram_dma.hdma2,
            0xff53 => self.vram_dma.hdma3,
            0xff54 => self.vram_dma.hdma4,
            0xff55 => self.vram_dma.hdma5,
            // This register allows to read/write data to the CGBs Background Palette Memory,
            // addressed through Register FF68. Each color is defined by two bytes (Bit
            // 0-7 in first byte).     Bit 0-4   Red Intensity   (00-1F)
            //     Bit 5-9   Green Intensity (00-1F)
            //     Bit 10-14 Blue Intensity  (00-1F)
            // Much like VRAM, Data in Palette Memory cannot be read/written during the time when
            // the LCD Controller is reading from it. (That is when the STAT register
            // indicates Mode 3). Note: Initially all background colors are initialized
            // as white.
            0xff68 => self.color_pal.bgpi,
            0xff69 => self.color_pal.bgp[(self.color_pal.bgpi & 0x3f) as usize],
            0xff6a => self.color_pal.obpi,
            0xff6b => self.color_pal.obp[(self.color_pal.obpi & 0x3f) as usize],
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9fff => self.vram.write(addr, data),
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00] = data,
            0xff40 => {
                //println!("lcdc = {:08b}", data);
                self.lcdc = data;
                if self.lcdc & 0x80 == 0 {
                    self.clear();
                    self.mode = Mode::HBlank;
                    self.line.ly = 0;
                }
            }
            0xff41 => {
                //println!("stat = {:08b}", data);
                self.stat = data;
            }
            0xff42 => self.scroll.scy = data,
            0xff43 => self.scroll.scx = data,
            0xff44 => {
                // The LY indicates the vertical line to which the present data
                // is transferred to the LCD Driver. The LY can take on any
                // value between 0 through 153. The values between 144 and 153
                // indicate the V-Blank period. Writing will reset the counter.
                self.line.ly = 0;
            }
            0xff45 => self.line.lyc = data,
            0xff4a => self.win.wy = data,
            0xff4b => self.win.wx = data,
            0xff47 => self.pal.bgp = data,
            0xff48 => self.pal.obp0 = data,
            0xff49 => self.pal.obp1 = data,
            0xff4f => self.vram.write(addr, data),
            0xff51 => self.vram_dma.hdma1 = data,
            0xff52 => self.vram_dma.hdma2 = data,
            0xff53 => self.vram_dma.hdma3 = data,
            0xff54 => self.vram_dma.hdma4 = data,
            0xff55 => {
                self.vram_dma.hdma5 = data;
                unimplemented!()
            }
            0xff68 => self.color_pal.bgpi = data,
            0xff69 => {
                self.color_pal.bgpi =
                    Self::write_color_pal(&mut self.color_pal.bgp[..], self.color_pal.bgpi, data)
            }
            0xff6a => self.color_pal.obpi = data,
            0xff6b => {
                self.color_pal.obpi =
                    Self::write_color_pal(&mut self.color_pal.obp[..], self.color_pal.obpi, data)
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cartridge::ZeroRom, dev::Device, mmu::Mmu};

    #[test]
    fn vram() {
        let mut mmu = Mmu::new(ZeroRom);

        mmu.write(0x8000, 1);
        mmu.write(0x9fff, 2);

        assert_eq!(1, mmu.read(0x8000));
        assert_eq!(2, mmu.read(0x9fff));
    }

    #[test]
    fn oam() {
        let mut mmu = Mmu::new(ZeroRom);

        mmu.write(0xfe00, 1);
        mmu.write(0xfe9f, 2);

        assert_eq!(1, mmu.read(0xfe00));
        assert_eq!(2, mmu.read(0xfe9f));
    }

    #[test]
    fn registers() {
        let mut mmu = Mmu::new(ZeroRom);

        mmu.write(0xff42, 1);
        mmu.write(0xff43, 2);
        mmu.write(0xff44, 3);
        mmu.write(0xff45, 4);
        mmu.write(0xff4a, 5);
        mmu.write(0xff4b, 6);
        mmu.write(0xff47, 7);
        mmu.write(0xff48, 8);
        mmu.write(0xff49, 9);

        assert_eq!(1, mmu.read(0xff42));
        assert_eq!(2, mmu.read(0xff43));
        // The LY indicates the vertical line to which the present data
        // is transferred to the LCD Driver. The LY can take on any
        // value between 0 through 153. The values between 144 and 153
        // indicate the V-Blank period. Writing will reset the counter.
        assert_eq!(0, mmu.read(0xff44));
        assert_eq!(4, mmu.read(0xff45));
        assert_eq!(5, mmu.read(0xff4a));
        assert_eq!(6, mmu.read(0xff4b));
        assert_eq!(7, mmu.read(0xff47));
        assert_eq!(8, mmu.read(0xff48));
        assert_eq!(9, mmu.read(0xff49));
    }
}
