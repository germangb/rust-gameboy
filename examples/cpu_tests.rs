use dmg::{cartridge::RomAndRam, ppu::palette::GRAYSCALE, Dmg, Mode};
use minifb::{Key, Window, WindowOptions};
use std::{
    mem, thread,
    thread::JoinHandle,
    time::{Duration, Instant},
};

fn main() {
    let tests = &[
        "roms/gb-test-roms/cpu_instrs/individual/02-interrupts.gb",
        "roms/gb-test-roms/cpu_instrs/individual/02-interrupts.gb",
        "roms/gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb",
        "roms/gb-test-roms/cpu_instrs/individual/04-op r,imm.gb",
        "roms/gb-test-roms/cpu_instrs/individual/05-op rp.gb",
        "roms/gb-test-roms/cpu_instrs/individual/06-ld r,r.gb",
        "roms/gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
        "roms/gb-test-roms/cpu_instrs/individual/08-misc instrs.gb",
        "roms/gb-test-roms/cpu_instrs/individual/09-op r,r.gb",
        "roms/gb-test-roms/cpu_instrs/individual/10-bit ops.gb",
        "roms/gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb",
    ];

    let mut threads = Vec::new();

    for (idx, test) in tests.iter().enumerate() {
        threads.push(thread::spawn(move || cpu_test(test, idx)));
    }

    threads
        .into_iter()
        .map(JoinHandle::join)
        .collect::<Result<Vec<_>, _>>()
        .expect("Error running tests");
}

fn cpu_test(file: &str, idx: usize) {
    let idx = idx as isize;
    let mut dmg = Dmg::new(
        RomAndRam::from_bytes(std::fs::read(file).unwrap()),
        Mode::GB,
    );
    let opts = WindowOptions::default();
    let mut window = Window::new("window", 160, 144, opts).unwrap();
    let x = idx % 5;
    let y = idx / 5;
    window.set_position((160 + 64) * x, (144 + 64) * y);
    while window.is_open() && !window.is_key_released(Key::Escape) {
        let begin = Instant::now();
        dmg.emulate_frame();
        let elapsed = begin.elapsed();
        let buffer = dmg.mmu().ppu().buffer();
        unsafe {
            window
                .update_with_buffer(mem::transmute(&buffer[..]), 160, 144)
                .unwrap();
        }
        let wait = Duration::new(0, 1_000_000_000 / 60);
        if elapsed < wait {
            thread::sleep(wait - elapsed);
        }
    }
}
