use dmg::{
    cartridge::RomOnly,
    device::Device,
    joypad::{
        Btn::{Select, Start, A, B},
        Dir::{Down, Left, Right, Up},
        Key::{Btn, Dir},
    },
    Dmg,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{mem, thread, time::Duration};

fn main() {
    let mut opt = WindowOptions::default();
    opt.scale = Scale::X2;
    let mut window = Window::new("Window", 160, 144, opt).unwrap();

    let mut dmg = Dmg::new(RomOnly::tetris());

    while window.is_open() {
        if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
            println!("if={:08b}", dmg.mmu().read(0xff0f));
            println!("ie={:08b}", dmg.mmu().read(0xffff));
            println!("stat={:08b}", dmg.mmu().read(0xff41));
            println!("lcdc={:08b}", dmg.mmu().read(0xff40));
            println!("if={:08b}", dmg.mmu().read(0xff0f));
            println!("ie={:08b}", dmg.mmu().read(0xffff));
            println!("{:?}", dmg.cpu());
        }

        let joypad = &[
            Dir(Down),
            Dir(Up),
            Dir(Left),
            Dir(Right),
            Btn(Start),
            Btn(Select),
            Btn(B),
            Btn(A),
        ];
        let keys = &[
            Key::Down,
            Key::Up,
            Key::Left,
            Key::Right,
            Key::Enter,
            Key::RightShift,
            Key::X,
            Key::Z,
        ];

        for (j, k) in joypad.iter().zip(keys) {
            if window.is_key_down(*k) {
                dmg.mmu_mut().joypad_mut().press(j);
            } else {
                dmg.mmu_mut().joypad_mut().release(j);
            }
        }

        dmg.emulate_frame();

        unsafe {
            let buffer = dmg.mmu().ppu().buffer();
            window
                .update_with_buffer(mem::transmute(&buffer[..]), 160, 144)
                .unwrap();
        }

        thread::sleep(Duration::new(0, 1_000_000_000 / 60));
    }
}
