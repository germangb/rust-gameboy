use dmg::{cartridge::RomOnly, device::Device, mmu::Mmu};

#[test]
fn checksum() {
    let rom = RomOnly::tetris();

    let mut res = 0x19u8;
    for addr in 0x134..=0x14d {
        res = res.wrapping_add(rom.read(addr as u16));
    }
    assert_eq!(0, res);
}

#[test]
fn checksum_mmu() {
    let mmu = Mmu::new(RomOnly::tetris());

    let mut res = 0x19u8;
    for addr in 0x134..=0x14d {
        res = res.wrapping_add(mmu.read(addr as u16));
    }
    assert_eq!(0, res);
}
