use dmg_lib::{cpu::Cpu, dev::Device, mmu::Mmu, ppu::palette::GRAYSCALE, Mode};
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const TIMEOUT: u64 = 4;

#[test]
fn boot() {
    let mut cpu = Cpu::default();
    let mut mmu = Mmu::with_cartridge_video_audio((), Mode::GB, (), ());

    let timeout = Arc::new(Mutex::new(Cell::new(false)));

    let to = timeout.clone();
    thread::spawn(move || {
        thread::sleep(Duration::new(TIMEOUT, 0));
        to.lock().unwrap().set(true);
    });

    let mut min_scy = 0xff;
    let mut max_scy = 0;

    mmu.write(0xff44, 144);

    while mmu.read(0xff50) == 0 {
        if timeout.lock().unwrap().get() {
            panic!("Timeout");
        }

        let cycles = cpu.step(&mut mmu);
        mmu.step(cycles);

        min_scy = min_scy.min(mmu.read(0xff42));
        max_scy = max_scy.max(mmu.read(0xff42));
    }

    assert_eq!(0, min_scy);
    assert_eq!(100, max_scy);
}
