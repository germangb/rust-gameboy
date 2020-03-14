use dmg::{cartridge::RomOnly, device::Device, mmu::Mmu};

#[test]
fn vram() {
    let mut mmu = Mmu::new(RomOnly::tetris());

    mmu.write(0x8000, 1);
    mmu.write(0x9fff, 2);

    assert_eq!(1, mmu.read(0x8000));
    assert_eq!(2, mmu.read(0x9fff));
}

#[test]
fn oam() {
    let mut mmu = Mmu::new(RomOnly::tetris());

    mmu.write(0xfe00, 1);
    mmu.write(0xfe9f, 2);

    assert_eq!(1, mmu.read(0xfe00));
    assert_eq!(2, mmu.read(0xfe9f));
}

#[test]
fn registers() {
    let mut mmu = Mmu::new(RomOnly::tetris());

    mmu.write(0xff42, 1);
    mmu.write(0xff43, 2);
    mmu.write(0xff44, 3);
    mmu.write(0xff45, 4);
    mmu.write(0xff4a, 5);
    mmu.write(0xff4b, 6);
    mmu.write(0xff47, 7);
    mmu.write(0xff48, 8);
    mmu.write(0xff49, 9);

    assert_eq!(1, mmu.read(0xff42));
    assert_eq!(2, mmu.read(0xff43));
    // The LY indicates the vertical line to which the present data
    // is transferred to the LCD Driver. The LY can take on any
    // value between 0 through 153. The values between 144 and 153
    // indicate the V-Blank period. Writing will reset the counter.
    assert_eq!(0, mmu.read(0xff44));
    assert_eq!(4, mmu.read(0xff45));
    assert_eq!(5, mmu.read(0xff4a));
    assert_eq!(6, mmu.read(0xff4b));
    assert_eq!(7, mmu.read(0xff47));
    assert_eq!(8, mmu.read(0xff48));
    assert_eq!(9, mmu.read(0xff49));
}
