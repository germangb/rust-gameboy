pub trait Mapped {
    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);

    fn read_signed(&self, addr: u16) -> i8 {
        unsafe { std::mem::transmute(self.read(addr)) }
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = u16::from(self.read(addr));
        let hi = u16::from(self.read(addr + 1)) << 8;
        hi | lo
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        let lo = data & 0xff;
        let hi = data >> 8;
        self.write(addr, lo as u8);
        self.write(addr + 1, hi as u8);
    }
}

impl<T: Mapped> Mapped for Box<T> {
    fn read(&self, addr: u16) -> u8 {
        <T as Mapped>::read(self, addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        <T as Mapped>::write(self, addr, data);
    }
}

#[cfg(test)]
mod tests {
    use crate::map::Mapped;

    #[test]
    fn words() {
        impl Mapped for [u8; 4] {
            fn read(&self, addr: u16) -> u8 {
                self[addr as usize]
            }

            fn write(&mut self, addr: u16, data: u8) {
                self[addr as usize] = data;
            }
        }

        let mut dev = [0u8; 4];

        dev.write_word(0, 0x1234);
        dev.write_word(2, 0xabcd);
        assert_eq!(0x1234, dev.read_word(0));
        assert_eq!(0xabcd, dev.read_word(2));
    }
}
