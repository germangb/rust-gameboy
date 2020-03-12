use crate::device::Device;

pub struct Timer {}

impl Timer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Device for Timer {
    fn read(&self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
