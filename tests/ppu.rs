use dmg::{cartridge::RomOnly, device::Device, mmu::Mmu};

#[test]
fn registers() {
    let mut mmu = Mmu::new(RomOnly::tetris());

    mmu.write(0xff42, 100);

    assert_eq!(100, mmu.read(0xff42));
}
