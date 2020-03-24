use crate::dev::Device;

pub struct Mbc5 {}

impl Device for Mbc5 {
    fn read(&self, _addr: u16) -> u8 {
        unimplemented!()
    }

    fn write(&mut self, _addr: u16, _data: u8) {
        unimplemented!()
    }
}
