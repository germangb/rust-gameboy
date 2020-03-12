use crate::device::Device;

pub struct Sound {}

impl Sound {
    pub fn new() -> Self {
        Self {}
    }
}

impl Device for Sound {
    fn read(&self, addr: u16) -> u8 {
        0
    }

    fn write(&mut self, addr: u16, data: u8) {}
}

#[cfg(test)]
mod tests {}
