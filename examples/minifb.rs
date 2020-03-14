use dmg::{
    cartridge::{RomOnly, RomRam},
    device::Device,
    joypad::{
        Btn::{Start, A},
        Key::Btn,
    },
    Dmg,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{mem, thread, time::Duration};

fn main() {
    let mut opt = WindowOptions::default();
    opt.scale = Scale::X2;
    let mut window = Window::new("Window", 160, 144, opt).unwrap();

    let mut dmg = Dmg::new(RomRam::dr_mario());

    while window.is_open() {
        dmg.emulate_frame();

        if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
            println!("if={:08b}", dmg.mmu().read(0xff0f));
            println!("ie={:08b}", dmg.mmu().read(0xffff));
            println!("stat={:08b}", dmg.mmu().read(0xff41));
            println!("lcdc={:08b}", dmg.mmu().read(0xff40));
            println!("if={:08b}", dmg.mmu().read(0xff0f));
            println!("ie={:08b}", dmg.mmu().read(0xffff));
            println!("{:?}", dmg.cpu());
        }

        if window.is_key_pressed(Key::A, KeyRepeat::No) {
            dmg.mmu_mut().joypad_mut().press(Btn(Start));
        }

        unsafe {
            let buffer = dmg.mmu().ppu().buffer();
            window
                .update_with_buffer(mem::transmute(&buffer[..]), 160, 144)
                .unwrap();
        }
        thread::sleep(Duration::new(0, 1_000_000_000 / 60));
    }
}
