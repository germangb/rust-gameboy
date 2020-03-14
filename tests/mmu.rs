use dmg::{cartridge::RomOnly, device::Device, mmu::Mmu};

#[test]
fn dma() {
    let mut mmu = Mmu::new(RomOnly::tetris());

    mmu.write(0xff50, 1);
    mmu.write(0xff46, 0);

    for addr in 0..=0x9f {
        let rom = mmu.read(addr as u16);
        let oam = mmu.read(0xfe00 | (addr as u16));
        assert_eq!(rom, oam);
    }
}
