static ROM: &[u8] = include_bytes!("roms/10-print.gb");

use dmg::{cartridge::RomAndRam, dev::Device, mmu::Mmu};

#[test]
fn checksum() {
    let mmu = Mmu::new(RomAndRam::from_bytes(ROM));

    let mut res = 0x19u8;
    for addr in 0x134..=0x14d {
        res = res.wrapping_add(mmu.read(addr as u16));
    }

    assert_eq!(0, res);
}

#[test]
#[ignore]
fn rom_only() {
    unimplemented!()
}

#[test]
#[ignore]
fn mbc1() {
    unimplemented!()
}

#[test]
#[ignore]
fn mbc3() {
    unimplemented!()
}
