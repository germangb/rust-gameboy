use dmg_driver_headless::ppu::{Buffer, HeadlessVideo};
use dmg_lib::{cartridge::Mbc3, Builder};
use std::time::{Duration, Instant};

static PPU: &[u8] = include_bytes!("cpu_instrs.bin");
static ROM: &[u8] = include_bytes!("cpu_instrs.gb");

#[test]
fn cpu_instrs() {
    let mut dmg = Builder::default()
        .with_cartridge(Mbc3::new(ROM))
        .with_video(HeadlessVideo::new())
        .build();

    let time = Instant::now();
    loop {
        if time.elapsed() > Duration::new(16, 0) {
            break;
        }
        dmg.emulate_frame();
    }

    let video = dmg.mmu().ppu().video();
    let buf = unsafe { std::slice::from_raw_parts(video.as_ptr(), 160 * 144 * 3) };

    assert_eq!(PPU, buf)
}
