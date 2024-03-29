use dmg_lib::ppu::{palette::Color, Video};
use std::mem;

pub type Buffer = [[Color; 160]; 144];

pub struct HeadlessVideo {
    front: Box<Buffer>,
    back: Box<Buffer>,
}

impl HeadlessVideo {
    pub fn new() -> Self {
        Self { front: Box::new([[[0, 0, 0]; 160]; 144]),
               back: Box::new([[[0, 0, 0]; 160]; 144]) }
    }

    pub fn buffer(&self) -> &Buffer {
        self.front.as_ref()
    }

    fn swap(&mut self) {
        mem::swap(&mut self.front, &mut self.back);
    }
}

impl Video for HeadlessVideo {
    fn draw_video(&mut self, pixels: &[[[u8; 3]; 160]; 144]) {
        unimplemented!()
    }
}
