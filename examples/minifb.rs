use dmg::{
    cartridge::{Cartridge, Mbc1, Mbc3, ZeroRom},
    joypad::{
        Btn::*,
        Dir::*,
        Key::{Btn, Dir},
    },
    ppu::palette::{Palette, NINTENDO_GAMEBOY_BLACK_ZERO},
    Dmg,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{
    mem, thread,
    time::{Duration, Instant},
};

const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;
const ROM: &[u8] =
    include_bytes!("../roms/Legend of Zelda, The - Link's Awakening DX (U) (V1.2) [C][!].gbc");

fn main() {
    let cartridge = Mbc3::from_bytes(ROM);

    println!("{:?}", cartridge.cgb());

    let mut dmg = Dmg::new(cartridge);
    dmg.mmu_mut().ppu_mut().set_palette(PALETTE);

    if dmg.mmu().cartridge().cgb().is_some() {
        dmg.boot_cgb();
    } else {
        dmg.boot();
    }

    let mut opt = WindowOptions::default();
    opt.scale = Scale::X2;
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
            let buffer = dmg.mmu().ppu().buffer();
            window
                .update_with_buffer(mem::transmute(&buffer[..]), 160, 144)
                .unwrap();
        }

        let elapsed = begin.elapsed();
        let wait = Duration::new(0, 1_000_000_000 / 60);
        if elapsed < wait {
            thread::sleep(wait - elapsed);
        }
    }
}
