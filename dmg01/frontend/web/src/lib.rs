pub use dmg_driver_wasm::ppu::WasmVideoOutput;
use dmg_lib::{
    cartridge::{Mbc1, Mbc5},
    joypad::{Btn, Dir, Key},
    ppu::palette::DMG,
    Builder, Mode,
};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
static ROM: &[u8] = include_bytes!("../../../tests/gb-test-roms/cpu_instrs/cpu_instrs.gb");

/// WebAssembly-enabled emulator.
#[wasm_bindgen]
pub struct Dmg(dmg_lib::Dmg<Mbc5, WasmVideoOutput, ()>);

#[wasm_bindgen]
pub fn init_wasm() {
    console_log::init_with_level(log::Level::Debug).expect("Error initializing log");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
impl Dmg {
    pub fn new(video: WasmVideoOutput) -> Self {
        let cart = Mbc5::new(ROM);
        let mut dmg = Builder::default()
            .with_mode(Mode::CGB)
            .with_video(video)
            .with_cartridge(cart)
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
