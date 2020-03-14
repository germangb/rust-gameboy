use dmg::{cartridge::RomOnly, cpu::Cpu, device::Device, mmu::Mmu};

#[test]
#[ignore]
fn interrupts() {
    let mut cpu = Cpu::default();
    let mut mmu = Mmu::new(RomOnly::tetris());

    mmu.write(0xff50, 1);
    mmu.write(0xffff, 0x10); // joypad

    assert_eq!(0, cpu.reg().pc)
}
