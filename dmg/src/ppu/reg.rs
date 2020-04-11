use crate::{
    map::Mapped,
    ppu::palette::{Color, GRAYSCALE},
};

const COLOR_PAL_SIZE: usize = 0x40;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Scroll {
    pub scy: u8,
    pub scx: u8,
}

impl Default for Scroll {
    fn default() -> Self {
        Self { scy: 0, scx: 0 }
    }
}

impl Mapped for Scroll {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff42 => self.scy,
            0xff43 => self.scx,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff42 => self.scy = data,
            0xff43 => self.scx = data,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Window {
    pub wy: u8,
    pub wx: u8,
}

impl Window {
    /// Returns the top-left corner of the window within the LCD display.
    /// Equivalent to `[wy, wx-7]`.
    ///
    /// The bounds of the window are defined by (wy, wx-7) being the top-left
    /// corner, and (143, 159) the bottom-right.
    pub fn lcd_bounds(self) -> [isize; 2] {
        [self.wy as isize, self.wx as isize - 7]
    }
}

impl Default for Window {
    fn default() -> Self {
        Self { wy: 0, wx: 0 }
    }
}

impl Mapped for Window {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff4a => self.wy = data,
            0xff4b => self.wx = data,
            _ => panic!(),
        }
    }
}

/// GB mode palette registers.
#[derive(Debug, Clone, Copy)]
pub struct Pal {
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    color_pal: [Color; 4],
}

impl Default for Pal {
    fn default() -> Self {
        Self {
            bgp: 0,
            obp0: 0,
            obp1: 0,
            color_pal: GRAYSCALE,
        }
    }
}

impl Pal {
    /// Set a custom set of 4 colors.
    /// If none is specified, 4 shades of gray are used.
    pub fn set_color_pal(&mut self, pal: [Color; 4]) {
        self.color_pal = pal;
    }

    /// Returns the cursom 4-color palette.
    /// If none is specified via [`Pal::set_color_pal`], it's four shades of
    /// gray.
    ///
    /// [`Pal::set_color_pal`]: #
    pub fn color_pal(&mut self) -> &[Color; 4] {
        &self.color_pal
    }

    /// Returns the color index given its index.
    /// The returned index may thenbe used to index a 4-color palette.
    pub fn bg_color(&self, index: usize) -> Color {
        let pal = self.bgp as usize;
        self.color_pal[pal >> (2 * index) & 0x3]
    }

    /// Returns the BG palette.
    pub fn bg_pal(&self) -> [Color; 4] {
        [
            self.bg_color(0),
            self.bg_color(1),
            self.bg_color(2),
            self.bg_color(3),
        ]
    }

    /// Returns the OB palette.
    pub fn ob_pal(&self, obp: usize) -> [Color; 4] {
        [
            self.obp_color(obp, 0),
            self.obp_color(obp, 1),
            self.obp_color(obp, 2),
            self.obp_color(obp, 3),
        ]
    }

    /// Return the color index from the given sprite index.
    ///
    /// # Panics
    /// Panics if `obp` is neither 0 nor 1.
    pub fn obp_color(&self, obp: usize, index: usize) -> Color {
        let pal = match obp {
            0 => self.obp0,
            1 => self.obp1,
            _ => panic!(),
        } as usize;
        self.color_pal[pal >> (2 * index) & 0x3]
    }

    /// Returns the color of the LCD when it's turned off (color index 0).
    pub fn clear_color(&self) -> Color {
        self.bg_color(0)
    }
}

impl Mapped for Pal {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff47 => self.bgp = data,
            0xff48 => self.obp0 = data,
            0xff49 => self.obp1 = data,
            _ => panic!(),
        }
    }
}

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
    pub bgp: [u8; COLOR_PAL_SIZE],
    pub obp: [u8; COLOR_PAL_SIZE],
}

impl Default for ColorPal {
    fn default() -> Self {
        Self {
            bgpi: 0,
            obpi: 0,
            bgp: [0xff; COLOR_PAL_SIZE],
            obp: [0xff; COLOR_PAL_SIZE],
        }
    }
}

impl ColorPal {
    /// Return a BG color palette.
    ///
    /// # Panics
    /// Panics if `palette` >= 8.
    pub fn bg_pal(&self, palette: usize) -> [Color; 4] {
        [
            self.bg_pal_color(palette, 0),
            self.bg_pal_color(palette, 1),
            self.bg_pal_color(palette, 2),
            self.bg_pal_color(palette, 3),
        ]
    }

    /// Return an OB color palette.
    ///
    /// # Panics
    /// Panics if `palette` >= 8.
    pub fn ob_pal(&self, palette: usize) -> [Color; 4] {
        [
            self.ob_pal_color(palette, 0),
            self.ob_pal_color(palette, 1),
            self.ob_pal_color(palette, 2),
            self.ob_pal_color(palette, 3),
        ]
    }

    /// Return one of the 4 colors of the given palette.
    /// There are a total of 8 color palettes with 4 colors each.
    ///
    /// # Panic
    /// Panics if `palette >= 8` or `color >= 4`
    pub fn bg_pal_color(&self, palette: usize, color: usize) -> Color {
        Self::pal_color(&self.bgp, palette, color)
    }

    /// Return a color from a color palette from the OB palettes.
    /// There are a total of 8 palettes with 4 colors each.
    ///
    /// # Panic
    /// Panics if `palette >= 8` or `color >= 4`
    pub fn ob_pal_color(&self, palette: usize, color: usize) -> Color {
        Self::pal_color(&self.obp, palette, color)
    }

    pub fn pal_color(pal: &[u8; COLOR_PAL_SIZE], palette: usize, color: usize) -> Color {
        assert!(palette < COLOR_PAL_SIZE / 8);
        assert!(color < 4);
        let pal_offset = 8 * palette;
        let pal = &pal[pal_offset..pal_offset + 8];
        let color_offset = color * 2;
        let color: u16 =
            u16::from(pal[color_offset as usize]) | u16::from(pal[color_offset as usize + 1]) << 8;
        [
            (0xff * (color & 0x1f) / 0x1f) as u8,
            (0xff * ((color >> 5) & 0x1f) / 0x1f) as u8,
            (0xff * ((color >> 10) & 0x1f) / 0x1f) as u8,
        ]
    }

    /// Returns the color of the LCD when it's turned off (white).
    pub fn clear_color(&self) -> Color {
        [0xff, 0xff, 0xff]
    }
}

impl Mapped for ColorPal {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff68 => self.bgpi,
            0xff69 => self.bgp[(self.bgpi & 0x3f) as usize],
            0xff6a => self.obpi,
            0xff6b => self.obp[(self.obpi & 0x3f) as usize],
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff68 => self.bgpi = data,
            0xff69 => self.bgpi = write_color_pal(&mut self.bgp[..], self.bgpi, data),
            0xff6a => self.obpi = data,
            0xff6b => self.obpi = write_color_pal(&mut self.obp[..], self.obpi, data),
            _ => panic!(),
        }
    }
}

// TODO run some tests. There are bugs in CGB mode
// This register allows to read/write data to the CGBs Background Palette
// Memory, addressed through Register FF68. Each color is defined by two bytes
// (Bit 0-7 in first byte).     Bit 0-4   Red Intensity   (00-1F)
//     Bit 5-9   Green Intensity (00-1F)
//     Bit 10-14 Blue Intensity  (00-1F)
// Much like VRAM, Data in Palette Memory cannot be read/written during the time
// when the LCD Controller is reading from it. (That is when the STAT register
// indicates Mode 3). Note: Initially all background colors are initialized
// as white.
fn write_color_pal(pal: &mut [u8], mut idx: u8, data: u8) -> u8 {
    pal[(idx & 0x3f) as usize] = data;
    if idx & 0x80 != 0 {
        idx += 1;
        idx &= 0xbf;
    }
    idx
}

/// Line registers.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Line {
    pub ly: u8,
    pub lyc: u8,
}

impl Default for Line {
    fn default() -> Self {
        Self { ly: 0, lyc: 0 }
    }
}

pub(crate) const STAT_VBLANK_FLAG: u8 = 0x10;
pub(crate) const STAT_HBLANK_FLAG: u8 = 0x08;
pub(crate) const STAT_SEARCH_FLAG: u8 = 0x20;
pub(crate) const STAT_LYC_LY_FLAG: u8 = 0x40;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum StatMode {
    HBlank = 0x00,
    VBlank = 0x01,
    Search = 0x02,
    Pixels = 0x03,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum TileMapAddr {
    X9c00 = 0x9c00,
    X9800 = 0x9800,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum TileDataAddr {
    X8000 = 0x8000,
    X8800 = 0x8800,
}

/// LCDC and STAT registers.
pub struct LcdcStat {
    // Bit 7 - LCD Display Enable             (0=Off, 1=On)
    // Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    // Bit 5 - Window Display Enable          (0=Off, 1=On)
    // Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    // Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    // Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    // Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    // Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
    pub lcdc: u8,
    // Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    // Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    // Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    // Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    // Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
    // Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
    pub stat: u8,
}

impl Default for LcdcStat {
    fn default() -> Self {
        Self { lcdc: 0, stat: 0 }
    }
}

impl LcdcStat {
    pub(crate) fn stat_set_mode(&mut self, mode: StatMode) {
        self.stat &= !0x3;
        self.stat |= mode as u8;
    }

    /// Returns 8 or 16 depending on the current OBJ size mode (bit 2).
    pub fn lcdc_ob_size(&self) -> u8 {
        if self.lcdc & 0x4 == 0 {
            8
        } else {
            16
        }
    }

    /// Location of the BG tile map.
    pub fn bg_tile_map(&self) -> TileMapAddr {
        if self.lcdc & 0x8 != 0 {
            TileMapAddr::X9c00
        } else {
            TileMapAddr::X9800
        }
    }

    /// Location of the BG & Window tile data.
    pub fn bg_win_tile_data(&self) -> TileDataAddr {
        if self.lcdc & 0x10 != 0 {
            TileDataAddr::X8000
        } else {
            TileDataAddr::X8800
        }
    }

    /// Location of the Window tile map.
    pub fn win_tile_map(&self) -> TileMapAddr {
        if self.lcdc & 0x40 != 0 {
            TileMapAddr::X9c00
        } else {
            TileMapAddr::X9800
        }
    }
}

impl Mapped for LcdcStat {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff40 => self.lcdc,
            0xff41 => self.stat,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff40 => self.lcdc = data,
            0xff41 => {
                self.stat &= 0x7;
                self.stat |= data & 0xf8;
            }
            _ => panic!(),
        }
    }
}
