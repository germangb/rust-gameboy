use dmg_lib::ppu::{Color, VideoOutput};
use wasm_bindgen::prelude::*;

const BUFFER: usize = 160 * 144 * 4;

#[wasm_bindgen]
pub struct CanvasVideoOutput {
    ctx: web_sys::CanvasRenderingContext2d,
    buf: [u8; BUFFER],
}

#[wasm_bindgen]
impl CanvasVideoOutput {
    pub fn with_context(ctx: web_sys::CanvasRenderingContext2d) -> Self {
        Self {
            ctx,
            buf: [0; BUFFER],
        }
    }
}

impl VideoOutput for CanvasVideoOutput {
    fn render_line(&mut self, line: usize, pixels: &[Color; 160]) {
        let offset = 160 * 4 * line;
        for (i, [r, g, b]) in pixels.iter().enumerate() {
            let [r, g, b, a] = [*r, *g, *b, 0xff];
            let offset = offset + 4 * i;
            self.buf[offset] = r;
            self.buf[offset + 1] = g;
            self.buf[offset + 2] = b;
            self.buf[offset + 3] = a;
        }

        if line == 143 {
            let clamped = wasm_bindgen::Clamped(&mut self.buf[..]);
            let image_data = web_sys::ImageData::new_with_u8_clamped_array(clamped, 160)
                .expect("Error creating image data");
            self.ctx
                .put_image_data(&image_data, 0.0, 0.0)
                .expect("Error writing image to canvas");
        }
    }
}
