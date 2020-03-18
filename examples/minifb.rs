use dmg::{
    cartridge::{Mbc1, Mbc3, RomOnly},
    device::Device,
    joypad::{
        Btn::*,
        Dir::*,
        Key::{Btn, Dir},
    },
    ppu::palette::{
        Color, ANDRADE_GAMEBOY, GRAYSCALE, HARSHGREEN, ICE_CREAM_GB, LINKS_AWAKENING_SGB,
        NINTENDO_GAMEBOY_BLACK_ZERO, NOSTALGIA, RUSTIC_GB,
    },
    Dmg,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{
    fs::File,
    io::Write,
    mem, thread,
    time::{Duration, Instant},
};

fn main() {
    let mut opt = WindowOptions::default();
    opt.scale = Scale::X2;
    let mut window = Window::new("Window", 160, 144, opt).unwrap();

    let mut dmg = Dmg::new(RomOnly::print10_demo());

    dmg.mmu_mut()
        .ppu_mut()
        .set_palette(NINTENDO_GAMEBOY_BLACK_ZERO);
    dmg.boot();

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
