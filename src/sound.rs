use crate::{device::Device, interrupts::Interrupts};
use std::{cell::RefCell, rc::Rc};

pub struct Sound {
    #[allow(dead_code)]
    int: Rc<RefCell<Interrupts>>,
}

impl Sound {
    pub fn new(int: Rc<RefCell<Interrupts>>) -> Self {
        Self { int }
    }
}

impl Device for Sound {
    fn read(&self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _data: u8) {}
}

#[cfg(test)]
mod tests {}
