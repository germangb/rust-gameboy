use crate::{
    int::{Flag, Flag::LCDCStat},
    map::Mapped,
    ppu::{
        oam::{Entry, Oam},
        palette::{Color, Palette},
        reg::{LcdcStat, STAT_HBLANK, STAT_LYC_LY, STAT_SEARCH, STAT_VBLANK},
    },
    vram::VRam,
    Mode,
};
use reg::{ColorPal, Line, Pal, Scroll, Window};
use std::mem;

pub mod oam;
pub mod palette;
pub mod reg;

const SEARCH: u64 = 80; //  80 dots (19 us)
const PIXELS: u64 = (168 + 291) / 2; // 168 to 291 cycles (40 to 60 us) depending on sprite count
const HBLANK: u64 = (85 + 208) / 2; // 85 to 208 dots (20 to 49 us) depending on previous mode 3 duration
const VBLANK: u64 = 4560; // 4560 dots (1087 us, 10 scanlines)

#[derive(Debug, Clone, Copy)]
enum LcdMode {
    HBlank = 0x00,
    VBlank = 0x01,
    Search = 0x02,
    Pixels = 0x03,
}

/// Display scanline renderer.
pub trait Video {
    fn render_line(&mut self, line: usize, pixels: &[Color; 160]);
}

impl Video for () {
    fn render_line(&mut self, _: usize, _: &[Color; 160]) {}
}

pub struct Ppu<V: Video> {
    video: V,
    mode: Mode,
    dots: u64,
    palette: Palette,
    buffer: [Color; 160],
    vram: VRam,
    oam: Oam,
    lcd_mode: LcdMode,
    lcdc_stat: LcdcStat,
    scroll: Scroll,
    line: Line,
    win: Window,
    pal: Pal,
    color_pal: ColorPal,
    vblank_int: Option<Flag>,
    lcdc_int: Option<Flag>,
}

impl<V: Video> Ppu<V> {
    pub fn new(mode: Mode, output: V) -> Self {
        Self {
            dots: 0,
            video: output,
            mode,
            palette: palette::GRAYSCALE,
            buffer: [[0xff, 0xff, 0xff]; 160],
            vram: VRam::default(),
            oam: Oam::default(),
            lcd_mode: LcdMode::HBlank,
            lcdc_stat: LcdcStat::default(),
            scroll: Scroll::default(),
            line: Line::default(),
            win: Window::default(),
            pal: Pal::default(),
            color_pal: ColorPal::default(),
            vblank_int: None,
            lcdc_int: None,
        }
    }

    pub(crate) fn take_vblank_int(&mut self) -> Option<Flag> {
        self.vblank_int.take()
    }

    pub(crate) fn take_lcdc_int(&mut self) -> Option<Flag> {
        self.lcdc_int.take()
    }

    /// Get color palette (GB mode only)
    pub fn gb_palette(&self) -> &Palette {
        &self.palette
    }

    /// Set color palette (GB mode only)
    pub fn set_gb_palette(&mut self, pal: Palette) {
        self.palette = pal;
    }

    pub fn video(&self) -> &V {
        &self.video
    }

    pub fn video_mut(&mut self) -> &mut V {
        &mut self.video
    }

    pub fn step(&mut self, cycles: u64) {
        if self.lcdc_stat.lcdc & 0x80 == 0 {
            return;
        }

        self.dots += cycles;

        let mut line = self.line.ly;

        match (self.lcd_mode, self.next_lcd_mode()) {
            (LcdMode::Search, LcdMode::Search) => {}
            (LcdMode::Search, LcdMode::Pixels) => {
                self.dots %= SEARCH;
                self.lcd_mode = LcdMode::Pixels;
            }

            (LcdMode::Pixels, LcdMode::Pixels) => { /* TODO dot by dot */ }
            (LcdMode::Pixels, LcdMode::HBlank) => {
                self.render_line(line, 0, 160);
                self.dots %= PIXELS;
                self.lcd_mode = LcdMode::HBlank;
                if self.lcdc_stat.stat & STAT_HBLANK != 0 {
                    self.request_lcdc();
                }
            }

            (LcdMode::HBlank, LcdMode::HBlank) => {}
            (LcdMode::HBlank, LcdMode::Search) if line == 143 => {
                self.dots %= HBLANK;
                self.lcd_mode = LcdMode::VBlank;
                self.request_vblank();
                if self.lcdc_stat.stat & STAT_VBLANK != 0 {
                    self.request_lcdc();
                }
                line = 144;
            }
            (LcdMode::HBlank, LcdMode::Search) => {
                self.dots %= HBLANK;
                self.lcd_mode = LcdMode::Search;
                if self.lcdc_stat.stat & STAT_SEARCH != 0 {
                    self.request_lcdc();
                }
                line += 1;
                // search 10 visible sprites in new line
                if self.lcdc_stat.lcdc & 0x2 != 0 {
                    let height = if self.lcdc_stat.lcdc & 0x4 != 0 {
                        16
                    } else {
                        8
                    };
                    self.oam.search(line, height);
                }
            }

            (LcdMode::VBlank, LcdMode::Search) => {
                self.dots %= VBLANK;
                self.lcd_mode = LcdMode::Search;
                if self.lcdc_stat.stat & STAT_SEARCH != 0 {
                    self.request_lcdc();
                }
                line = 0;
            }
            (LcdMode::VBlank, LcdMode::VBlank) => {
                const DOTS_LINE: u64 = 456;

                line = 144 + (self.dots / DOTS_LINE) as u8;

                // TODO maybe set LY=0 at some point in VBlank
                // if line == 153 && self.dots % DOTS_LINE > DOTS_LINE / 2 {
                //     line = 0;
                // }
            }

            _ => panic!(),
        }

        // LY=LYC
        if line == self.line.lyc {
            if line != self.line.ly && self.lcdc_stat.stat & STAT_LYC_LY != 0 {
                self.request_lcdc();
            }
            self.lcdc_stat.stat |= 0b0000_0100;
        } else {
            self.lcdc_stat.stat &= 0b1111_1011;
        }

        self.line.ly = line;
        self.lcdc_stat.stat_set_mode(self.lcd_mode as u8);
    }

    fn next_lcd_mode(&self) -> LcdMode {
        match self.lcd_mode {
            LcdMode::Search if self.dots >= SEARCH => LcdMode::Pixels,
            LcdMode::Pixels if self.dots >= PIXELS => LcdMode::HBlank,
            LcdMode::HBlank if self.dots >= HBLANK => LcdMode::Search,
            LcdMode::VBlank if self.dots >= VBLANK => LcdMode::Search,
            _ => self.lcd_mode,
        }
    }

    fn request_vblank(&mut self) {
        self.vblank_int = Some(Flag::VBlank);
    }

    fn request_lcdc(&mut self) {
        self.lcdc_int = Some(Flag::LCDCStat);
    }

    fn clear_video(&mut self) {
        let color = match self.mode {
            Mode::GB => self.palette[0],
            Mode::CGB => [0xff, 0xff, 0xff],
        };
        mem::replace(&mut self.buffer, [color; 160]);
        for i in 0..144 {
            self.video.render_line(i, &self.buffer);
        }
    }

    fn render_line(&mut self, ly: u8, offset: usize, dots: usize) {
        let bg = match self.mode {
            Mode::GB => self.lcdc_stat.lcdc & 0x1 != 0,
            Mode::CGB => true,
        };
        if bg {
            self.render_bg(ly, offset, dots);
        }
        let win = self.lcdc_stat.lcdc & 0x20 != 0;
        if win {
            self.render_win(ly, offset, dots)
        }
        let obj = self.lcdc_stat.lcdc & 0x2 != 0;
        if obj {
            self.render_sprites(ly, offset, dots)
        }
        self.video.render_line(ly as usize, &self.buffer);
    }

    // fetch pixel color from a given coordinate
    // coordinate is relative to the tilemap origin
    #[rustfmt::skip]
    fn point_color(&mut self, y: usize, x: usize, tile_map: u16, tile_data: u16) -> Color {
        // look up tile index
        let tile_map_idx = 32 * (y / 8) + (x / 8);
        let tile_map_offset = tile_map as usize - 0x8000;
        let tile = self.vram.bank(0)[tile_map_offset + tile_map_idx];
        let flags = self.vram.bank(1)[tile_map_offset + tile_map_idx];
        // look up tile pixel
        let mut col = 7 - (x & 7);
        let mut row = y & 7;
        if let Mode::CGB = self.mode {
            // flip tiles in CGB mode
            if flags & 0x20 != 0 { col = 7 - col }
            if flags & 0x40 != 0 { row = 7 - row }
        }
        let offset = match tile_data {
            0x8000 => 16 * (tile as usize) + row * 2,
            0x8800 => {
                let tile: i8 = unsafe { mem::transmute(tile) };
                let tile = (tile as isize + 128) as usize;
                0x800 + 16 * tile + row * 2
            }
            _ => panic!(),
        };
        let bank = match self.mode {
            Mode::GB => 0,
            Mode::CGB => (flags >> 3) & 0x1,
        };
        let lo = self.vram.bank(bank as usize)[offset] >> (col as u8) & 0x1;
        let hi = self.vram.bank(bank as usize)[offset + 1] >> (col as u8) & 0x1;
        let pal_idx = lo | (hi << 1);
        // return pixel color
        match self.mode {
            Mode::GB => {
                let pal = self.pal.bgp;
                let shade = (pal >> (2 * pal_idx)) & 0x3;
                self.palette[shade as usize]
            }
            Mode::CGB => unimplemented!(),
        }
    }

    fn render_win(&mut self, ly: u8, offset: usize, dots: usize) {
        let Window { wy, wx } = self.win;
        // The window becomes visible (if enabled) when positions are set in
        // range WX=0..166, WY=0..143. A position of WX=7, WY=0 locates the
        // window at upper left, it is then completely covering normal
        // background.
        let ly = ly as i16;
        let wy = wy as i16;
        let wx = wx as i16 - 7;
        // window below current line
        // or too far to the right
        let offset = offset as i16;
        let dots = dots as i16;
        if ly < wy || offset + dots < wx {
            return;
        }
        for lcd_x in wx..(offset + dots) {
            let map = self.lcdc_stat.win_tile_map();
            let data = self.lcdc_stat.bg_win_tile_data();
            self.buffer[lcd_x as usize] =
                self.point_color((ly - wy) as usize, (lcd_x - wx) as usize, map, data);
        }
    }

    fn render_bg(&mut self, ly: u8, offset: usize, dots: usize) {
        let Scroll { scx, scy } = self.scroll;
        for lcd_x in offset..(offset + dots) {
            let y = ly.wrapping_add(scy) as usize;
            let x = (lcd_x as u8).wrapping_add(scx) as usize;
            let map = self.lcdc_stat.bg_tile_map();
            let data = self.lcdc_stat.bg_win_tile_data();
            self.buffer[lcd_x] = self.point_color(y, x, map, data);
        }
    }

    fn render_sprites(&mut self, ly: u8, offset: usize, dots: usize) {
        // TODO
    }
}

impl<V: Video> Mapped for Ppu<V> {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.vram.read(addr),
            0xfe00..=0xfe9f => self.oam.read(addr),
            0xff40 | 0xff41 => self.lcdc_stat.read(addr),
            //0xff40 => self.lcdc,
            //0xff41 => self.stat,
            0xff42 | 0xff43 => self.scroll.read(addr),
            0xff44 => self.line.ly,
            0xff45 => self.line.lyc,
            0xff4a | 0xff4b => self.win.read(addr),
            0xff47..=0xff49 => self.pal.read(addr),
            0xff4f => self.vram.read(addr),
            0xff68..=0xff6b => self.color_pal.read(addr),
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9fff => self.vram.write(addr, data),
            0xfe00..=0xfe9f => self.oam.write(addr, data),
            0xff40 | 0xff41 => {
                let lcdc = self.lcdc_stat.lcdc;
                self.lcdc_stat.write(addr, data);

                // LCD display disabled
                if lcdc & 0x80 != 0 && self.lcdc_stat.lcdc & 0x80 == 0 {
                    self.dots = 0;
                    self.line.ly = 0;
                    self.lcd_mode = LcdMode::HBlank;
                    self.lcdc_stat.stat_set_mode(self.lcd_mode as u8);

                    // clear video output
                    self.clear_video();
                }
            }
            0xff42 | 0xff43 => self.scroll.write(addr, data),
            0xff44 => {
                // to The LY indicates the vertical line to which the present
                // data is transferred to the LCD Driver. The LY can take on any
                // value between 0 through 153. The values between 144 and 153
                // indicate the V-Blank period. Writing will reset the counter.

                self.dots = 0;
                self.line.ly = 0;
                self.lcd_mode = LcdMode::Search;
                self.lcdc_stat.stat_set_mode(self.lcd_mode as u8);
            }
            0xff45 => self.line.lyc = data,
            0xff4a | 0xff4b => self.win.write(addr, data),
            0xff47..=0xff49 => self.pal.write(addr, data),
            0xff4f => self.vram.write(addr, data),
            0xff68..=0xff6b => self.color_pal.write(addr, data),
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{map::Mapped, mmu::Mmu, Mode};

    #[test]
    fn vram() {
        let mut mmu = Mmu::<_, _, ()>::new(Mode::GB, (), ());

        mmu.write(0x8000, 1);
        mmu.write(0x9fff, 2);

        assert_eq!(1, mmu.read(0x8000));
        assert_eq!(2, mmu.read(0x9fff));
    }

    #[test]
    fn oam() {
        let mut mmu = Mmu::<_, _, ()>::new(Mode::GB, (), ());

        mmu.write(0xfe00, 1);
        mmu.write(0xfe9f, 2);

        assert_eq!(1, mmu.read(0xfe00));
        assert_eq!(2, mmu.read(0xfe9f));
    }

    #[test]
    fn registers() {
        let mut mmu = Mmu::<_, _, ()>::new(Mode::GB, (), ());

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
