use crate::{
    dev::Device,
    interrupts::Flag,
    ppu::palette::{Color, Palette},
    vram::VideoRam,
    Mode,
};
use std::{mem, slice};

pub mod palette;

// Mode 0 is present between 201-207 clks, 2 about 77-83 clks, and 3 about
// 169-175 clks. A complete cycle through these states takes 456 clks. VBlank
// lasts 4560 clks. A complete screen refresh occurs every 70224 clks.)
pub const HBLANK_CYCLES: u64 = 51 * 4;
pub const OAM_CYCLES: u64 = 20 * 4;
pub const PIXEL_TRANSFER_CYCLES: u64 = 43 * 4;
pub const VBLANK_CYCLES: u64 = (OAM_CYCLES + PIXEL_TRANSFER_CYCLES + HBLANK_CYCLES) * 10;

const OAM_SIZE: usize = 0xa0;
const PAL_SIZE: usize = 0x40;

pub trait VideoOutput {
    fn render_line(&mut self, line: usize, pixels: &[Color; 160]);
}

impl VideoOutput for () {
    fn render_line(&mut self, _: usize, _: &[Color; 160]) {}
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    HBlank = 0x00,
    VBlank = 0x01,
    OAM = 0x02,
    PixelTransfer = 0x03,
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
pub struct Scroll {
    pub scy: u8,
    pub scx: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Window {
    pub wy: u8,
    pub wx: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pal {
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColorPal {
    // This register is used to address a byte in the CGBs Background Palette Memory. Each two byte
    // in that memory define a color value. The first 8 bytes define Color 0-3 of Palette 0 (BGP0),
    // and so on for BGP1-7.
    //     Bit 0-5   Index (00-3F)
    //     Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
    // Data can be read/written to/from the specified index address through Register FF69. When the
    // Auto Increment Bit is set then the index is automatically incremented after each <write> to
    // FF69. Auto Increment has no effect when <reading> from FF69, so the index must be manually
    // incremented in that case.
    pub bgpi: u8,
    pub obpi: u8,
    pub bgp: [u8; PAL_SIZE],
    pub obp: [u8; PAL_SIZE],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Line {
    pub ly: u8,
    pub lyc: u8,
}

pub struct Ppu<V: VideoOutput> {
    output: V,
    mode: Mode,
    cycles: u64,
    palette: Palette,
    buffer: [Color; 160],
    index: [u8; 160],
    vram: VideoRam,
    oam: [u8; OAM_SIZE],
    state: State,
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
    pal: Pal,
    color_pal: ColorPal,
    // Interrupt requests
    vblank_int: Option<Flag>,
    lcdc_int: Option<Flag>,
}

impl<V: VideoOutput> Ppu<V> {
    pub fn with_mode_and_video(mode: Mode, output: V) -> Self {
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
            // initialize colors to black
            bgp: [0x00; PAL_SIZE],
            obp: [0x00; PAL_SIZE],
        };
        let vram = VideoRam::default();
        let lcdc = 0x00;
        let stat = 0x00;
        let buffer = [[0xff, 0xff, 0xff]; 160];
        let index = [0; 160];
        let oam = [0; OAM_SIZE];
        let state = State::HBlank;
        Self {
            cycles: 0,
            output,
            mode,
            palette,
            buffer,
            index,
            vram,
            oam,
            state,
            lcdc,
            stat,
            scroll,
            line,
            win,
            pal,
            color_pal,
            vblank_int: None,
            lcdc_int: None,
        }
    }

    pub fn take_vblank_int(&mut self) -> Option<Flag> {
        self.vblank_int.take()
    }

    pub fn take_lcdc_int(&mut self) -> Option<Flag> {
        self.lcdc_int.take()
    }

    pub fn line(&self) -> &Line {
        &self.line
    }

    /// Get color palette (GB mode only)
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Set color palette (GB mode only)
    pub fn set_palette(&mut self, pal: Palette) {
        self.palette = pal;
    }

    /// Return the list of OAM entries.
    pub fn oam_entries(&self) -> &[OamEntry] {
        unsafe { slice::from_raw_parts(self.oam.as_ptr() as _, 40) }
    }

    /// Return the mutable list of OAM entries.
    pub fn oam_entries_mut(&mut self) -> &mut [OamEntry] {
        unsafe { slice::from_raw_parts_mut(self.oam.as_ptr() as _, 40) }
    }

    pub fn video_output(&self) -> &V {
        &self.output
    }

    pub fn video_output_mut(&mut self) -> &mut V {
        &mut self.output
    }

    pub fn window(&self) -> &Window {
        &self.win
    }

    pub fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    pub fn pal(&self) -> &Pal {
        &self.pal
    }

    pub fn color_pal(&self) -> &ColorPal {
        &self.color_pal
    }

    pub(crate) fn step(&mut self, cycles: u64) {
        self.cycles += cycles;

        let mut line = self.line.ly;

        match (self.state, self.next_state()) {
            (State::OAM, State::PixelTransfer) => {
                self.state = State::PixelTransfer;
                // self.render_line(line, 0, self.cycles as usize);
            }
            (State::PixelTransfer, State::HBlank) => {
                self.state = State::HBlank;
                self.render_line(line, 0, 160);
                self.output.render_line(line as usize, &self.buffer);

                // update line
                line += 1;

                // hblank interrupt
                if self.stat & 0x8 != 0 {
                    self.request_lcdc();

                    #[cfg(feature = "logging")]
                    log::info!(target: "ppu", "LCDC Status (HBLANK) interrupt requested");
                }
            }
            (State::HBlank, State::OAM) if line == 144 => {
                self.state = State::VBlank;

                // LCD STAT vblank interrupt
                if self.stat & 0x10 != 0 {
                    self.request_lcdc();

                    #[cfg(feature = "logging")]
                    log::info!(target: "ppu", "LCDC Status (VBLANK) interrupt requested");
                }

                // vblank interrupt
                self.request_vblank();

                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "VBLANK interrupt requested");
            }
            (State::HBlank, State::OAM) | (State::VBlank, State::OAM) => {
                self.state = State::OAM;

                // OAM interrupt
                if self.stat & 0x20 != 0 {
                    self.request_lcdc();
                }
            }
            (State::PixelTransfer, State::PixelTransfer) => {
                // TODO Pixel FIFO
                // let offset = self.cycles - cycles;
                // self.render_line(line, offset as usize, cycles as usize);
            }
            (State::OAM, State::OAM) | (State::HBlank, State::HBlank) => { /* carry on */ }
            (State::VBlank, State::VBlank) => {
                if self.line.ly == 153 && self.cycles >= OAM_CYCLES + PIXEL_TRANSFER_CYCLES / 2 {
                    // not setting the line counter causes some top-row-glitches on games that rely
                    // on this interrupt for effects (link's awakening & Batman)
                    line = 0;
                } else if self.line.ly != 0 {
                    let vb_line =
                        self.cycles / (OAM_CYCLES + PIXEL_TRANSFER_CYCLES + HBLANK_CYCLES);

                    line = 144 + vb_line as u8;
                }
            }
            _ => panic!(),
        }

        // line interrupt
        if line != self.line.ly {
            self.line.ly = line;

            // check new line interrupt
            if self.stat & 0x40 != 0 && self.line.ly == self.line.lyc {
                self.request_lcdc();

                #[cfg(feature = "logging")]
                log::info!(
                    target: "ppu",
                    "LCDC Status (LYC=LY) interrupt requested. LY = {} LYC = {}",
                    self.line.ly,
                    self.line.lyc
                );
            }
        }

        if self.line.ly == self.line.lyc {
            self.stat |= 0x4;
        } else {
            self.stat &= !0x4;
        }

        self.stat &= 0xfc;
        self.stat |= self.state as u8;
    }

    fn request_vblank(&mut self) {
        self.vblank_int = Some(Flag::VBlank);
    }

    fn request_lcdc(&mut self) {
        self.lcdc_int = Some(Flag::LCDCStat);
    }

    fn next_state(&mut self) -> State {
        match self.state {
            State::OAM => {
                if self.cycles >= OAM_CYCLES {
                    self.cycles %= OAM_CYCLES;
                    State::PixelTransfer
                } else {
                    State::OAM
                }
            }
            State::PixelTransfer => {
                if self.cycles >= PIXEL_TRANSFER_CYCLES {
                    self.cycles %= PIXEL_TRANSFER_CYCLES;
                    State::HBlank
                } else {
                    State::PixelTransfer
                }
            }
            State::HBlank => {
                if self.cycles >= HBLANK_CYCLES {
                    self.cycles %= HBLANK_CYCLES;
                    State::OAM
                } else {
                    State::HBlank
                }
            }
            State::VBlank => {
                if self.cycles >= VBLANK_CYCLES {
                    self.cycles %= VBLANK_CYCLES;
                    State::OAM
                } else {
                    State::VBlank
                }
            }
        }
    }

    fn write_color_pal(pal: &mut [u8], mut idx: u8, data: u8) -> u8 {
        pal[(idx & 0x3f) as usize] = data;
        if idx & 0x80 != 0 {
            idx += 1;
            idx &= 0xbf;
        }
        idx
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

    fn render_line(&mut self, ly: u8, offset: usize, len: usize) {
        if self.lcdc & 0x80 == 0 {
            return;
        }

        let bg = self.lcdc & 0x1 != 0;
        let obj = self.lcdc & 0x2 != 0;
        let win = self.lcdc & 0x20 != 0;
        match self.mode {
            Mode::GB => {
                if bg {
                    self.render_bg(ly, offset, len);
                }
            }
            Mode::CGB => {
                self.render_bg(ly, offset, len);
            }
        }
        if win {
            self.render_win(ly);
        }
        if obj {
            self.render_sprites(ly);
        }
    }

    fn clear_buffer(&mut self) {
        let color = match self.mode {
            Mode::GB => self.palette[0],
            Mode::CGB => [0xff, 0xff, 0xff],
        };
        mem::replace(&mut self.buffer, [color; 160]);
        for i in 0..144 {
            self.output.render_line(i, &self.buffer);
        }
    }

    fn render_win(&mut self, ly: u8) {
        let Window { wy, wx } = self.win;
        let Pal { bgp, .. } = self.pal;
        if ly < wy || wx >= 160 {
            return;
        }
        let gb_pal = bgp;
        let win_tile_map = self.win_tile_map();
        let bg_win_tile_data = self.bg_win_tile_data();
        for pix in wx.max(7)..=166 {
            let lcd_y = u16::from(ly - wy);
            let lcd_x = u16::from(pix - wx);
            let pixel = (pix - 7) as usize;

            let tile_map_idx = (32u16 * (lcd_y / 8) + (lcd_x / 8)) as usize;
            let bank_0 = self.vram.bank(0);
            let bank_1 = self.vram.bank(1);
            let (tile, flags) = match win_tile_map {
                TileMap::X9c00 => (bank_0[0x1c00 + tile_map_idx], bank_1[0x1c00 + tile_map_idx]),
                TileMap::X9800 => (bank_0[0x1800 + tile_map_idx], bank_1[0x1800 + tile_map_idx]),
            };
            let col = 7 - (lcd_x & 0x7) as u8;
            let lin = lcd_y & 0x7;

            let tile_data_bank = if flags & 0x8 != 0 { bank_1 } else { bank_0 };
            let pal_idx = match bg_win_tile_data {
                TileData::X8000 => {
                    let offset = 16 * tile as usize + lin as usize * 2;
                    let lo = tile_data_bank[offset] >> col & 0x1;
                    let hi = tile_data_bank[offset + 1] >> col & 0x1;
                    ((hi << 1) | lo) as usize
                }
                TileData::X8800 => {
                    let tile: i8 = unsafe { mem::transmute(tile) };
                    let tile = (tile as i16 + 128) as u16;

                    let offset = 0x800 + 16 * tile as usize + lin as usize * 2;
                    let lo = tile_data_bank[offset] >> col & 0x1;
                    let hi = tile_data_bank[offset + 1] >> col & 0x1;
                    ((hi << 1) | lo) as usize
                }
            };

            match self.mode {
                Mode::GB => {
                    let col_idx = (gb_pal >> (2 * pal_idx as u8)) & 0x3;
                    self.buffer[pixel] = self.palette[col_idx as usize];
                }
                Mode::CGB => {
                    let gbc_pal = (flags & 0x7) as usize;
                    let gbc_pal = &self.color_pal.bgp[8 * gbc_pal..8 * gbc_pal + 8];
                    let color: u16 =
                        u16::from(gbc_pal[2 * pal_idx]) | u16::from(gbc_pal[2 * pal_idx + 1]) << 8;
                    let r = (0xff * (color & 0x1f) / 0x1f) as u8;
                    let g = (0xff * ((color >> 5) & 0x1f) / 0x1f) as u8;
                    let b = (0xff * ((color >> 10) & 0x1f) / 0x1f) as u8;
                    self.buffer[pixel] = [r, g, b];
                }
            }
        }
    }

    fn render_bg(&mut self, ly: u8, offset: usize, len: usize) {
        let Pal { bgp: gb_pal, .. } = self.pal;
        let bg_tile_map = self.bg_tile_map();
        let bg_win_tile_data = self.bg_win_tile_data();
        let Scroll { scy, scx } = self.scroll;
        for pixel in offset..(offset + len).min(160) {
            let lcd_y = scy.wrapping_add(ly) as u16;
            let lcd_x = (pixel as u8).wrapping_add(scx) as u16;

            let tile_map_idx = (32u16 * (lcd_y / 8) + (lcd_x / 8)) as usize;
            let bank_0 = self.vram.bank(0);
            let bank_1 = self.vram.bank(1); // CGB only (tile flags)

            let (tile, flags) = match bg_tile_map {
                TileMap::X9c00 => (bank_0[0x1c00 + tile_map_idx], bank_1[0x1c00 + tile_map_idx]),
                TileMap::X9800 => (bank_0[0x1800 + tile_map_idx], bank_1[0x1800 + tile_map_idx]),
            };

            let mut col = 7 - (lcd_x & 0x7) as u8;
            let mut lin = lcd_y & 0x7;
            // Flip tiles (CGB only)
            if flags & 0x20 != 0 {
                col = 7 - col
            }
            if flags & 0x40 != 0 {
                lin = 7 - lin
            }

            // On CGB mode the tile data may be stored in either bank. In GB mode, on the
            // first one, as the second one is not used.
            let tile_data_bank = if flags & 0x8 != 0 { bank_1 } else { bank_0 };
            let pal_idx = match bg_win_tile_data {
                TileData::X8000 => {
                    let offset = 16 * tile as usize + lin as usize * 2;
                    let lo = tile_data_bank[offset] >> col & 0x1;
                    let hi = tile_data_bank[offset + 1] >> col & 0x1;
                    ((hi << 1) | lo) as usize
                }
                TileData::X8800 => {
                    let tile: i8 = unsafe { mem::transmute(tile) };
                    let tile = (tile as i16 + 128) as u16;
                    let offset = 0x800 + 16 * tile as usize + lin as usize * 2;
                    let lo = tile_data_bank[offset] >> col & 0x1;
                    let hi = tile_data_bank[offset + 1] >> col & 0x1;
                    ((hi << 1) | lo) as usize
                }
            };

            self.index[pixel] = pal_idx as u8;

            match self.mode {
                Mode::GB => {
                    let col_idx = (gb_pal >> (2 * pal_idx as u8)) & 0x3;
                    self.buffer[pixel] = self.palette[col_idx as usize];
                }
                Mode::CGB => {
                    let gbc_pal = (flags & 0x7) as usize;
                    let gbc_pal = &self.color_pal.bgp[8 * gbc_pal..8 * gbc_pal + 8];
                    let color: u16 =
                        u16::from(gbc_pal[2 * pal_idx]) | u16::from(gbc_pal[2 * pal_idx + 1]) << 8;
                    self.buffer[pixel] = [
                        (0xff * (color & 0x1f) / 0x1f) as u8,
                        (0xff * ((color >> 5) & 0x1f) / 0x1f) as u8,
                        (0xff * ((color >> 10) & 0x1f) / 0x1f) as u8,
                    ];
                }
            }
        }
    }

    fn render_sprites(&mut self, ly: u8) {
        let Pal { obp0, obp1, .. } = self.pal;
        // let mut entries = self.oam_entries().to_vec();
        // entries.reverse();
        // entries.sort_by_key(|o| o.xpos);

        // TODO workaround borrow checker avoid clone
        let entries =
            unsafe { slice::from_raw_parts(self.oam_entries().as_ptr(), self.oam_entries().len()) };

        for oam in entries {
            let tile = u16::from(oam.tile);
            let xpos = i16::from(oam.xpos);
            let ypos = i16::from(oam.ypos);

            let obj_w = 8;
            let obj_h = if self.lcdc & 0x4 != 0 { 16 } else { 8 };
            let obj_y = ly as i16;

            if obj_y < ypos - 16
                || obj_h == 16 && ly as i16 >= ypos
                || obj_h == 8 && ly as i16 >= ypos - 8
            {
                continue;
            }

            let x_flip = oam.flag & 0x20 != 0;
            let y_flip = oam.flag & 0x40 != 0;
            let gb_pal = if oam.flag & 0x10 != 0 { obp1 } else { obp0 };
            let gbc_pal = (oam.flag & 0x7) as usize;

            for obj_x in xpos - 8..xpos {
                if obj_x >= 0 && obj_x < 160 && obj_y >= 0 && obj_y < 144 {
                    let pixel = obj_x as usize;
                    let mut lin = (obj_y - (ypos - 16)) as u16;
                    let mut col = 7 - (obj_x - (xpos - 8)) as u8;
                    if x_flip {
                        col = obj_w - col - 1
                    }
                    if y_flip {
                        lin = obj_h - lin - 1;
                    }

                    let bank_0 = self.vram.bank(0);
                    let bank_1 = self.vram.bank(1);
                    let obj_bank = if oam.flag & 0x8 == 0 { bank_0 } else { bank_1 };

                    let offset = 16 * tile as usize + lin as usize * 2;
                    let lo = (obj_bank[offset] >> col) & 0x1;
                    let hi = (obj_bank[offset + 1] >> col) & 0x1;
                    // discard transparent pixels
                    let pal = ((hi << 1) | lo) as usize;
                    if pal == 0 {
                        continue;
                    }

                    // sprite behind BG colors 1-3
                    let behind_bg = oam.flag & 0x80 != 0;
                    if behind_bg && self.index[pixel] != 0 {
                        continue;
                    }

                    match self.mode {
                        Mode::GB => {
                            let col_idx = (gb_pal >> (2 * pal as u8)) & 0x3;
                            self.buffer[pixel] = self.palette[col_idx as usize];
                        }
                        Mode::CGB => {
                            let gbc_pal = &self.color_pal.obp[8 * gbc_pal..8 * gbc_pal + 8];
                            let color: u16 =
                                u16::from(gbc_pal[2 * pal]) | u16::from(gbc_pal[2 * pal + 1]) << 8;
                            self.buffer[pixel] = [
                                (0xff * (color & 0x1f) / 0x1f) as u8,
                                (0xff * ((color >> 5) & 0x1f) / 0x1f) as u8,
                                (0xff * ((color >> 10) & 0x1f) / 0x1f) as u8,
                            ];
                        }
                    }
                }
            }
        }
    }
}

impl<V: VideoOutput> Device for Ppu<V> {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.vram.read(addr),
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00],
            0xff40 => self.lcdc,
            0xff41 => self.stat,
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
                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "LCDC = {:#08b}", data);

                self.lcdc = data;
                if self.lcdc & 0x80 == 0 {
                    self.clear_buffer();
                    self.state = State::HBlank;
                    self.line.ly = 0;
                }
            }
            0xff41 => {
                self.stat &= 0x3;
                self.stat |= data & 0xfc;

                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "STAT = {:#08b}", data);
            }
            0xff42 => self.scroll.scy = data,
            0xff43 => self.scroll.scx = data,
            0xff44 => {
                // The LY indicates the vertical line to which the present data
                // is transferred to the LCD Driver. The LY can take on any
                // value between 0 through 153. The values between 144 and 153
                // indicate the V-Blank period. Writing will reset the counter.
                self.line.ly = 0;
                self.cycles = 0;
                self.state = State::OAM;
            }
            0xff45 => {
                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "LYC = {}", data);

                self.line.lyc = data;
            }
            0xff4a => self.win.wy = data,
            0xff4b => self.win.wx = data,
            0xff47 => {
                self.pal.bgp = data;

                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "BGP = {:#02x}", data);
            }
            0xff48 => {
                self.pal.obp0 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "OBP0 = {:#02x}", data);
            }
            0xff49 => {
                self.pal.obp1 = data;

                #[cfg(feature = "logging")]
                log::info!(target: "ppu", "OBP1 = {:#02x}", data);
            }
            0xff4f => self.vram.write(addr, data),
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
    use crate::{dev::Device, mmu::Mmu, Mode};

    #[test]
    fn vram() {
        let mut mmu = Mmu::with_cartridge_and_video((), Mode::GB, ());

        mmu.write(0x8000, 1);
        mmu.write(0x9fff, 2);

        assert_eq!(1, mmu.read(0x8000));
        assert_eq!(2, mmu.read(0x9fff));
    }

    #[test]
    fn oam() {
        let mut mmu = Mmu::with_cartridge_and_video((), Mode::GB, ());

        mmu.write(0xfe00, 1);
        mmu.write(0xfe9f, 2);

        assert_eq!(1, mmu.read(0xfe00));
        assert_eq!(2, mmu.read(0xfe9f));
    }

    #[test]
    fn registers() {
        let mut mmu = Mmu::with_cartridge_and_video((), Mode::GB, ());

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
