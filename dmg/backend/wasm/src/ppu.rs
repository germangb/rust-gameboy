use dmg_lib::ppu::{palette::Color, Video};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

const BUFFER_SIZE: usize = 160 * 144 * 4;

#[wasm_bindgen]
pub struct WasmVideo {
    ctx: CanvasRenderingContext2d,
    buf: [u8; BUFFER_SIZE],
}

#[wasm_bindgen]
impl WasmVideo {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self {
            ctx,
            buf: [0; BUFFER_SIZE],
        }
    }
}

impl Video for WasmVideo {
    fn draw_video(&mut self, pixels: &[Color; 160]) {
        let clamped = wasm_bindgen::Clamped(&mut self.buf[..]);
        let image_data = web_sys::ImageData::new_with_u8_clamped_array(clamped, 160)
            .expect("Error creating image data");
        self.ctx
            .put_image_data(&image_data, 0.0, 0.0)
            .expect("Error writing image to canvas");
    }
}
