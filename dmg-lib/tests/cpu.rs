use dmg_lib::{
    apu::AudioOutput,
    cartridge::{Cartridge, Mbc1},
    ppu::{palette::GRAYSCALE, Color, VideoOutput},
    Dmg, Mode,
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
fn cpu_instrs() {
    test(Dmg::new(Mbc1::from_bytes(ROM), Mode::GB, (), ()));
}

fn test<C: Cartridge, V: VideoOutput, A: AudioOutput>(mut dmg: Dmg<C, V, A>) {
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
}
