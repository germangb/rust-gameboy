use crate::device::Device;

#[repr(u8)]
pub enum Flag {
    VBlank = 0x1,
    LCDStat = 0x2,
    Timer = 0x4,
    Serial = 0x8,
    Joypad = 0x10,
}

#[derive(Debug)]
pub struct Interrupts {
    flags: u8,
    enable: u8,
}

impl Default for Interrupts {
    fn default() -> Self {
        Self {
            enable: 0x0,
            flags: 0x0,
        }
    }
}

impl Interrupts {
    pub fn set(&mut self, flag: Flag) {
        self.flags |= flag as u8;
    }

    pub fn reset(&mut self, flag: Flag) {
        self.flags &= !(flag as u8);
    }

    pub fn is_enabled(&self, flag: Flag) -> bool {
        self.enable & (flag as u8) != 0
    }

    pub fn is_active(&self, flag: Flag) -> bool {
        self.flags & (flag as u8) != 0
    }
}

impl Device for Interrupts {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff0f => self.flags,
            0xffff => self.enable,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff0f => self.flags = data,
            0xffff => {
                //println!("IE={:08b}", data);
                self.enable = data
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        device::Device,
        interrupts::{Flag::*, Interrupts},
    };

    #[test]
    fn request() {
        let mut int = Interrupts::default();

        int.set(VBlank);
        assert_eq!(0b00001, int.read(0xff0f));
        int.set(LCDStat);
        assert_eq!(0b00011, int.read(0xff0f));
        int.set(Timer);
        assert_eq!(0b00111, int.read(0xff0f));
        int.reset(VBlank);
        int.set(Serial);
        assert_eq!(0b01110, int.read(0xff0f));
        int.reset(LCDStat);
        int.set(Joypad);
        assert_eq!(0b11100, int.read(0xff0f));
        int.reset(Timer);
        assert_eq!(0b11000, int.read(0xff0f));
        int.reset(Serial);
        assert_eq!(0b10000, int.read(0xff0f));
        int.reset(Joypad);
        assert_eq!(0b00000, int.read(0xff0f));
    }
}
