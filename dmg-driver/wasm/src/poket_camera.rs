use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlVideoElement};

use dmg_peripheral_camera::CameraSensor;

#[wasm_bindgen]
pub struct WasmCameraSensor {
    vid: HtmlVideoElement,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl WasmCameraSensor {
    pub fn with_video_and_context(vid: HtmlVideoElement, ctx: CanvasRenderingContext2d) -> Self {
        Self { vid, ctx }
    }
}

impl CameraSensor for WasmCameraSensor {
    fn capture(&mut self, buffer: &mut [[u8; 128]; 112]) {
        self.ctx
            .draw_image_with_html_video_element_and_dw_and_dh(&self.vid, 0.0, 0.0, 128.0, 112.0)
            .expect("Error writing video to canvas");
        let image_data = self
            .ctx
            .get_image_data(0.0, 0.0, 128.0, 112.0)
            .expect("Error getting image data");
        let data = image_data.data().0;
        for (i, row) in buffer.iter_mut().enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                let offset = (128 * i + j) * 4;
                let r = data[offset] as f64 / 255.0;
                let g = data[offset + 1] as f64 / 255.0;
                let b = data[offset + 2] as f64 / 255.0;
                let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                *col = (y * 255.0).min(255.0) as u8;
            }
        }
    }
}
