use dmg::{cartridge::RomOnly, cpu::Cpu, device::Device, mmu::Mmu};
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const TIMEOUT: u64 = 1;

#[test]
fn boot() {
    let mut cpu = Cpu::new();
    let mut mmu = Mmu::new(RomOnly::tetris());

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

        let _ = cpu.step(&mut mmu);

        min_scy = min_scy.min(mmu.read(0xff42));
        max_scy = max_scy.max(mmu.read(0xff42));
    }

    assert_eq!(0, min_scy);
    assert_eq!(100, max_scy);
}
