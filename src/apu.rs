use crate::{dev::Device, interrupts::Interrupts};
use std::{cell::RefCell, rc::Rc};

pub trait AudioOutput {}

pub struct Apu {
    #[allow(dead_code)]
    int: Rc<RefCell<Interrupts>>,
}

impl Apu {
    pub fn new(int: Rc<RefCell<Interrupts>>) -> Self {
        Self { int }
    }
}

impl Device for Apu {
    fn read(&self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _data: u8) {}
}

#[cfg(test)]
mod tests {}
