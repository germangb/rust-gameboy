use dmg_lib::ppu::{palette::Color, VideoOutput};
use sdl2::{
    pixels::PixelFormatEnum,
    render::{TextureAccess, WindowCanvas},
};
use std::{
    ops::{Deref, DerefMut},
    slice,
};

const PIXELS: usize = 160 * 144;

pub struct Sdl2VideoOutput {
    canvas: WindowCanvas,
    buffer: Box<[[Color; 160]; 144]>,
}

impl Deref for Sdl2VideoOutput {
    type Target = WindowCanvas;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for Sdl2VideoOutput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}

impl Sdl2VideoOutput {
    pub fn from_canvas(canvas: WindowCanvas) -> Self {
        Self {
            canvas,
            buffer: Box::new([[[0, 0, 0]; 160]; 144]),
        }
    }

    pub fn canvas(&self) -> &WindowCanvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut WindowCanvas {
        &mut self.canvas
    }
}

impl VideoOutput for Sdl2VideoOutput {
    fn render_line(&mut self, line: usize, pixels: &[Color; 160]) {
        std::mem::replace(&mut self.buffer[line], *pixels);
        if line == 143 {
            let texture_creator = self.canvas.texture_creator();
            let mut texture = texture_creator
                .create_texture(PixelFormatEnum::RGB24, TextureAccess::Static, 160, 144)
                .expect("Error creating SDL texture");

            unsafe {
                let slice = slice::from_raw_parts(self.buffer.as_ptr() as *const u8, PIXELS * 3);
                texture
                    .update(None, slice, 160 * 3)
                    .expect("Error updating SDL texture");
            };

            self.canvas.copy(&texture, None, None).unwrap();
        }
    }
}