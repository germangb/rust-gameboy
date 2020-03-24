use dmg_lib::ppu::{Color, VideoOutput};
use gl::types::*;

pub struct OpenGLVideoOutput {
    texture: GLuint,
}

impl VideoOutput for OpenGLVideoOutput {
    fn render_line(&mut self, line: usize, pixels: &[Color; 160]) {
        unimplemented!()
    }
}

impl Drop for OpenGLVideoOutput {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
        }
    }
}
