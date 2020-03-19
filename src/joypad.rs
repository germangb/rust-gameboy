use crate::{
    dev::Device,
    interrupts::{Flag, Interrupts},
};
use std::{cell::RefCell, rc::Rc};

const BTN: u8 = 0x10;
const DIR: u8 = 0x20;
const NONE: u8 = 0xf;

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Btn(Btn),
    Dir(Dir),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Btn {
    Start = 0x8,
    Select = 0x4,
    A = 0x2,
    B = 0x1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Dir {
    Down = 0x8,
    Up = 0x4,
    Left = 0x2,
    Right = 0x1,
}

pub struct Joypad {
    int: Rc<RefCell<Interrupts>>,
    joyp: u8,
    btn: u8,
    dir: u8,
}

impl Joypad {
    pub fn new(int: Rc<RefCell<Interrupts>>) -> Self {
        Self {
            int,
            joyp: 0x00,
            btn: NONE,
            dir: NONE,
        }
    }

    pub fn press(&mut self, key: Key) {
        let (btn, dir) = match key {
            Key::Btn(btn) => (self.btn & !(btn as u8), self.dir),
            Key::Dir(dir) => (self.btn, self.dir & !(dir as u8)),
        };

        // Joypad interrupt is requested when any of the above Input lines changes from
        // High to Low. Generally this should happen when a key becomes pressed
        // (provided that the button/direction key is enabled by above Bit4/5), however,
        // because of switch bounce, one or more High to Low transitions are usually
        // produced both when pressing or releasing a key.
        if self.btn != btn || self.dir != dir {
            self.int.borrow_mut().set(Flag::Joypad);
        }

        self.btn = btn;
        self.dir = dir;
    }

    pub fn release(&mut self, key: Key) {
        match key {
            Key::Btn(btn) => self.btn |= btn as u8,
            Key::Dir(dir) => self.dir |= dir as u8,
        }
    }

    fn dir(&self) -> u8 {
        self.dir & 0xf
    }

    fn btn(&self) -> u8 {
        self.btn & 0xf
    }
}

// The eight gameboy buttons/direction keys are arranged in form of a 2x4
// matrix. Select either button or direction keys by writing to this register,
// then read-out bit 0-3.
//
// Bit 7 - Not used
// Bit 6 - Not used
// Bit 5 - P15 Select Button Keys      (0=Select)
// Bit 4 - P14 Select Direction Keys   (0=Select)
// Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
// Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
// Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
// Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
impl Device for Joypad {
    fn read(&self, addr: u16) -> u8 {
        assert_eq!(0xff00, addr);
        match self.joyp & 0x30 {
            BTN => BTN | self.btn(),
            DIR => DIR | self.dir(),
            0x30 => 0x3f,
            0x0 => 0xf,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        assert_eq!(0xff00, addr);
        self.joyp = data;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dev::Device,
        interrupts::Interrupts,
        joypad::{Btn::*, Dir::*, Joypad, Key, BTN, DIR},
    };
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn joypad_never_0() {
        let int = Rc::new(RefCell::new(Interrupts::default()));
        let mut joypad = Joypad::new(int);

        assert_ne!(0, joypad.read(0xff00));

        joypad.write(0xff00, 0);

        assert_ne!(0, joypad.read(0xff00));

        joypad.press(Key::Dir(Down));
        joypad.press(Key::Dir(Up));
        joypad.press(Key::Dir(Left));
        joypad.press(Key::Dir(Right));
        joypad.press(Key::Btn(Select));
        joypad.press(Key::Btn(Start));
        joypad.press(Key::Btn(A));
        joypad.press(Key::Btn(B));

        joypad.write(0xff00, DIR);
        assert_ne!(0, joypad.read(0xff00));
        joypad.write(0xff00, BTN);
        assert_ne!(0, joypad.read(0xff00));
        joypad.write(0xff00, BTN | DIR);
        assert_ne!(0, joypad.read(0xff00));
        joypad.write(0xff00, 0);
        assert_ne!(0, joypad.read(0xff00));
    }

    #[test]
    fn joypad_select() {
        let int = Rc::new(RefCell::new(Interrupts::default()));
        let mut joypad = Joypad::new(int);

        joypad.press(Key::Btn(Select));
        joypad.press(Key::Btn(Start));

        joypad.write(0xff00, DIR);
        assert_eq!(DIR | 0xf, joypad.read(0xff00));
        joypad.press(Key::Dir(Down));
        assert_eq!(DIR | 0b0111, joypad.read(0xff00));
        joypad.press(Key::Dir(Up));
        assert_eq!(DIR | 0b0011, joypad.read(0xff00));
        joypad.press(Key::Dir(Left));
        assert_eq!(DIR | 0b0001, joypad.read(0xff00));

        joypad.write(0xff00, BTN);
        assert_eq!(BTN | 0b0011, joypad.read(0xff00));

        joypad.write(0xff00, DIR);
        assert_eq!(DIR | 0b0001, joypad.read(0xff00));
        joypad.press(Key::Dir(Right));
        assert_eq!(DIR | 0b0000, joypad.read(0xff00));

        joypad.write(0xff00, BTN);
        assert_eq!(BTN | 0b0011, joypad.read(0xff00));
        joypad.press(Key::Btn(A));
        assert_eq!(BTN | 0b0001, joypad.read(0xff00));
        joypad.press(Key::Btn(B));
        assert_eq!(BTN | 0b0000, joypad.read(0xff00));
    }
}
