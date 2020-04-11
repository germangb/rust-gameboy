use dmg_lib::ppu::{palette::Color, Video};
use sdl2::{
    pixels::PixelFormatEnum,
    render::{TextureAccess, WindowCanvas},
};
use std::{
    ops::{Deref, DerefMut},
    slice,
};

const PIXELS: usize = 160 * 144;

pub struct SdlVideo {
    canvas: WindowCanvas,
}

impl SdlVideo {
    pub fn new(canvas: WindowCanvas) -> Self {
        Self { canvas }
    }

    pub fn canvas(&self) -> &WindowCanvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut WindowCanvas {
        &mut self.canvas
    }
}

impl Video for SdlVideo {
    fn draw_video(&mut self, pixels: &[[Color; 160]; 144]) {
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture(PixelFormatEnum::RGB24, TextureAccess::Static, 160, 144)
            .expect("Error creating SDL texture");

        unsafe {
            let slice = slice::from_raw_parts(pixels.as_ptr() as *const u8, PIXELS * 3);
            texture
                .update(None, slice, 160 * 3)
                .expect("Error updating SDL texture");
        };

        self.canvas.copy(&texture, None, None).unwrap();
    }
}
