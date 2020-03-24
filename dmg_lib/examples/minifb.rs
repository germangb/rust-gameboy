use dmg_lib::{
    cartridge::{Cartridge, Mbc1, Mbc3, ZeroRom},
    joypad::{
        Btn::*,
        Dir::*,
        Key::{Btn, Dir},
    },
    ppu::{
        palette::{Palette, NINTENDO_GAMEBOY_BLACK_ZERO},
        VideoOutput,
    },
    Dmg, Mode,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{
    io::Cursor,
    mem, thread,
    time::{Duration, Instant},
};

const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;
const ROM: &[u8] = include_bytes!("../roms/Tetris-USA.gb");

struct Output {
    front: Vec<u32>,
    back: Vec<u32>,
}

impl VideoOutput for Output {
    fn render_line(&mut self, line: usize, pixels: &[[u8; 3]; 160]) {
        for (i, [r, g, b]) in pixels.iter().enumerate() {
            self.back[160 * line + i] = u32::from(*r) << 16 | u32::from(*g) << 8 | u32::from(*b);
        }
        if line == 143 {
            std::mem::swap(&mut self.front, &mut self.back);
        }
    }
}

fn main() {
    let output = Output {
        back: vec![0u32; 160 * 144],
        front: vec![0u32; 160 * 144],
    };

    let cart = Mbc3::from_bytes(ROM);
    let mut dmg = Dmg::new(cart, Mode::GB, output, ());
    dmg.mmu_mut()
        .ppu_mut()
        .set_palette(NINTENDO_GAMEBOY_BLACK_ZERO);
    dmg.boot();

    let mut opt = WindowOptions::default();
    opt.scale = Scale::X4;
    opt.resize = true;
    let mut window = Window::new("Window", 160, 144, opt).unwrap();

    while window.is_open() {
        let joy = &[
            Dir(Down),
            Dir(Up),
            Dir(Left),
            Dir(Right),
            Btn(Start),
            Btn(Select),
            Btn(B),
            Btn(A),
        ];
        let key = &[
            Key::Down,
            Key::Up,
            Key::Left,
            Key::Right,
            Key::Enter,
            Key::RightShift,
            Key::X,
            Key::Z,
        ];

        for (j, k) in joy.iter().zip(key) {
            if window.is_key_down(*k) {
                dmg.mmu_mut().joypad_mut().press(*j);
            } else {
                dmg.mmu_mut().joypad_mut().release(*j);
            }
        }

        if window.is_key_pressed(Key::Q, KeyRepeat::No) {
            eprintln!("{:#?}", dmg.cpu());
        }

        let begin = Instant::now();
        dmg.emulate_frame();

        unsafe {
            let output = dmg.mmu().ppu().video_output();
            window
                .update_with_buffer(mem::transmute(&output.front[..]), 160, 144)
                .unwrap();
        }

        let elapsed = begin.elapsed();
        let wait = Duration::new(0, 1_000_000_000 / 60);
        if elapsed < wait {
            thread::sleep(wait - elapsed);
        }
    }
}
