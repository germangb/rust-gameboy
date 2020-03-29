#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
#![deny(unused_variables)]
#![deny(unused_mut)]
#![deny(unused_imports)]
#![deny(clippy::style)]
#![deny(clippy::correctness)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
use dmg_camera::PoketCamera;
use dmg_lib::{
    joypad::{Btn, Dir, Key},
    ppu::palette::{Palette, NINTENDO_GAMEBOY_BLACK_ZERO},
    Builder, Mode,
};

pub use dmg_driver_wasm::{poket_camera::WasmCameraSensor, ppu::WasmVideoOutput};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const MODE: Mode = Mode::GB;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

/// WebAssembly-enabled emulator.
#[wasm_bindgen::prelude::wasm_bindgen]
pub struct Dmg(dmg_lib::Dmg<PoketCamera<WasmCameraSensor>, WasmVideoOutput, ()>);

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn init_log() {
    console_log::init_with_level(log::Level::Debug).expect("Error initializing log");
}

#[wasm_bindgen::prelude::wasm_bindgen]
impl Dmg {
    pub fn with_video_and_sensor(video: WasmVideoOutput, sensor: WasmCameraSensor) -> Self {
        let dmg = Builder::default()
            .with_mode(MODE)
            .with_palette(PALETTE)
            .with_video(video)
            .with_cartridge(PoketCamera::with_sensor(sensor))
            .build();
        Self(dmg)
    }

    pub fn emulate_frame(&mut self) {
        self.0.emulate_frame();
    }

    pub fn handle_key_down(&mut self, event: &web_sys::KeyboardEvent) {
        if let Some(key) = map_code_to_key(&event.code()) {
            let jpad = self.0.mmu_mut().joypad_mut();
            jpad.press(key)
        }
    }

    pub fn handle_key_up(&mut self, event: &web_sys::KeyboardEvent) {
        if let Some(key) = map_code_to_key(&event.code()) {
            let jpad = self.0.mmu_mut().joypad_mut();
            jpad.release(key)
        }
    }
}

fn map_code_to_key(code: &str) -> Option<Key> {
    match code {
        "KeyZ" => Some(Key::Btn(Btn::A)),
        "KeyX" => Some(Key::Btn(Btn::B)),
        "Enter" => Some(Key::Btn(Btn::Start)),
        "ShiftRight" => Some(Key::Btn(Btn::Select)),
        "ArrowLeft" => Some(Key::Dir(Dir::Left)),
        "ArrowRight" => Some(Key::Dir(Dir::Right)),
        "ArrowUp" => Some(Key::Dir(Dir::Up)),
        "ArrowDown" => Some(Key::Dir(Dir::Down)),
        _ => None,
    }
}
