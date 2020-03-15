use dmg::{cartridge::RomOnly, Dmg};
use minifb::{Key, Window, WindowOptions};
use std::{mem, thread, time::Duration};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    cpu_test("roms/cpu_instrs/individual/01-special.gb");
    cpu_test("roms/cpu_instrs/individual/02-interrupts.gb");
    cpu_test("roms/cpu_instrs/individual/02-interrupts.gb");
    cpu_test("roms/cpu_instrs/individual/03-op sp,hl.gb");
    cpu_test("roms/cpu_instrs/individual/04-op r,imm.gb");
    cpu_test("roms/cpu_instrs/individual/05-op rp.gb");
    cpu_test("roms/cpu_instrs/individual/06-ld r,r.gb");
    cpu_test("roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
    cpu_test("roms/cpu_instrs/individual/08-misc instrs.gb");
    cpu_test("roms/cpu_instrs/individual/09-op r,r.gb");
    cpu_test("roms/cpu_instrs/individual/10-bit ops.gb");
    cpu_test("roms/cpu_instrs/individual/11-op a,(hl).gb");
}

fn cpu_test(file: &str) {
    let mut dmg = Dmg::new(RomOnly::from_bytes(std::fs::read(file).unwrap()));
    let opts = WindowOptions::default();
    let mut window = Window::new("window", WIDTH, HEIGHT, opts).unwrap();
    while window.is_open() && !window.is_key_released(Key::Escape) {
        dmg.emulate_frame();
        let buffer = dmg.mmu().ppu().buffer();
        unsafe {
            window
                .update_with_buffer(mem::transmute(&buffer[..]), 160, 144)
                .unwrap();
        }
    }
}
