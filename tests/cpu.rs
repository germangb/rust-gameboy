use dmg::{
    cartridge::Mbc1,
    ppu::palette::{Color, GRAYSCALE},
    Dmg,
};
use std::{
    cell::Cell,
    mem,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

static ROM: &[u8] = include_bytes!("../roms/gb-test-roms/cpu_instrs/cpu_instrs.gb");

// display when all tests have passed
// uses GRAYSCALE palette
static PASS: &[u8] = include_bytes!("cpu.bin");

#[test]
fn cpu_instrs_no_boot() {
    let mut dmg = Dmg::new(Mbc1::from_bytes(ROM));

    // skip boot sequence
    dmg.boot();

    test(dmg);
}

#[test]
fn cpu_instrs() {
    test(Dmg::new(Mbc1::from_bytes(ROM)));
}

fn test(mut dmg: Dmg) {
    dmg.mmu_mut().ppu_mut().set_palette(GRAYSCALE);

    let timeout = Arc::new(Mutex::new(Cell::new(false)));

    let timeout_thread = timeout.clone();
    let to = thread::spawn(move || {
        thread::sleep(Duration::new(60, 0));
        timeout_thread.lock().unwrap().set(true);
    });

    while !timeout.lock().unwrap().get() {
        dmg.emulate_frame();
    }

    to.join().unwrap();

    let buffer = unsafe {
        let size = 160 * 144 * mem::size_of::<Color>();
        let ptr = dmg.mmu().ppu().buffer().as_ptr() as *const u8;

        std::slice::from_raw_parts(ptr, size)
    };

    assert_eq!(PASS, buffer);
}
