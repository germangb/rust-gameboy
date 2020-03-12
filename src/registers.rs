#[repr(u8)]
pub enum Flag {
    Z = 0x80,
    N = 0x40,
    H = 0x20,
    C = 0x10,
}

#[rustfmt::skip]
#[derive(Debug)]
pub struct Registers {
    pub a: u8, pub f: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Registers {
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            a: 0, f: 0,
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            pc: 0,
            sp: 0
        }
    }

    pub fn af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f)
    }

    pub fn bc(&self) -> u16 {
        u16::from(self.b) << 8 | u16::from(self.c)
    }

    pub fn de(&self) -> u16 {
        u16::from(self.d) << 8 | u16::from(self.e)
    }

    pub fn hl(&self) -> u16 {
        u16::from(self.h) << 8 | u16::from(self.l)
    }

    pub fn set_af(&mut self, af: u16) {
        self.a = (af >> 8) as u8;
        self.f = (af & 0xff) as u8;
    }

    pub fn set_bc(&mut self, bc: u16) {
        self.b = (bc >> 8) as u8;
        self.c = (bc & 0xff) as u8;
    }

    pub fn set_de(&mut self, de: u16) {
        self.d = (de >> 8) as u8;
        self.e = (de & 0xff) as u8;
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.h = (hl >> 8) as u8;
        self.l = (hl & 0xff) as u8;
    }

    pub fn is_flag(&self, flag: Flag) -> bool {
        self.f & (flag as u8) != 0
    }

    pub fn set_flag(&mut self, flag: Flag, b: bool) {
        if b {
            self.f |= flag as u8;
        } else {
            self.f &= !(flag as u8);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::registers::{
        Flag::{C, H, N, Z},
        Registers,
    };

    #[test]
    fn registers() {
        let mut reg = Registers {
            a: 0,
            f: 0,
            b: 0x45,
            c: 0x67,
            d: 0x89,
            e: 0xab,
            h: 0xcd,
            l: 0xef,
            pc: 0,
            sp: 0,
        };

        assert_eq!(0x4567, reg.bc());
        assert_eq!(0x89ab, reg.de());
        assert_eq!(0xcdef, reg.hl());

        reg.set_af(0x0123);
        assert_eq!(0x01, reg.a);
        assert_eq!(0x23, reg.f);
    }

    #[test]
    fn flags() {
        let mut reg = Registers::new();

        reg.set_flag(Z, true);
        reg.set_flag(N, false);
        reg.set_flag(H, true);
        reg.set_flag(C, false);

        assert_eq!(0xA0, reg.f);
        assert!(reg.is_flag(Z));
        assert!(!reg.is_flag(N));
        assert!(reg.is_flag(H));
        assert!(!reg.is_flag(C));
    }
}
