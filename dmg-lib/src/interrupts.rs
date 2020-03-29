use crate::dev::Device;

#[repr(u8)]
pub enum Flag {
    VBlank = 0x1,
    LCDCStat = 0x2,
    Timer = 0x4,
    Serial = 0x8,
    Joypad = 0x10,
}

#[derive(Debug)]
pub struct Interrupts {
    if_: u8,
    ie: u8,
}

impl Default for Interrupts {
    fn default() -> Self {
        Self { ie: 0x0, if_: 0x0 }
    }
}

impl Interrupts {
    /// Request an interrupt by setting the appropriate flag in the IF register.
    /// This may be done by the PPU, the TIMER, or the user code.
    pub fn set(&mut self, flag: Flag) {
        self.if_ |= flag as u8;
    }

    /// Returns true if the given flag is set in the IF register.
    pub fn is_enabled(&self, flag: Flag) -> bool {
        self.ie & (flag as u8) != 0
    }

    /// Returns true if the given flag is not set in the IF register.
    pub fn is_active(&self, flag: Flag) -> bool {
        self.if_ & (flag as u8) != 0
    }
}

impl Device for Interrupts {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff0f => self.if_,
            0xffff => self.ie,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff0f => self.if_ = data,
            0xffff => {
                if self.ie != data {
                    #[cfg(feature = "logging")]
                    log::info!("IE <- {:#08b}", data);
                }
                self.ie = data
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dev::Device,
        interrupts::{Flag::*, Interrupts},
    };

    #[test]
    fn request() {
        let mut int = Interrupts::default();

        int.set(VBlank);
        assert_eq!(0b00001, int.read(0xff0f));
        int.set(LCDCStat);
        assert_eq!(0b00011, int.read(0xff0f));
        int.set(Timer);
        assert_eq!(0b00111, int.read(0xff0f));
        int.reset(VBlank);
        int.set(Serial);
        assert_eq!(0b01110, int.read(0xff0f));
        int.reset(LCDCStat);
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
