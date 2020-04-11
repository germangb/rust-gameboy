pub use dmg_driver_wasm::ppu::WasmVideoOutput;
use dmg_lib::{
    cartridge,
    cartridge::{Controller, Mbc1, Mbc5},
    joypad::{Btn, Dir, Key},
    ppu::palette::DMG,
    Builder, Mode,
};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static ROM: &[u8] = include_bytes!("../../native/roms/Dr. Mario (World).gb");

/// WebAssembly-enabled emulator.
#[wasm_bindgen]
pub struct Dmg(dmg_lib::Dmg<Box<dyn Controller>, WasmVideoOutput, ()>);

#[wasm_bindgen]
pub fn init_wasm() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
impl Dmg {
    pub fn new(video: WasmVideoOutput) -> Self {
        let mut dmg = Builder::default()
            .with_mode(Mode::GB)
            .with_video(video)
            .with_cartridge(cartridge::from_bytes(ROM).unwrap())
            .build();
        dmg.mmu_mut().ppu_mut().pal_mut().set_color_pal(DMG);
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
