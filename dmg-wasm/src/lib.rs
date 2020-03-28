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
use crate::video::CanvasVideoOutput;
use dmg_lib::{
    cartridge::Mbc3,
    joypad::{
        Btn::*,
        Dir::*,
        Key,
        Key::{Btn, Dir},
    },
    ppu::palette::{Palette, NINTENDO_GAMEBOY_BLACK_ZERO},
    Builder, Mode,
};
use wasm_bindgen::prelude::*;

pub mod audio;
pub mod video;

type Mbc = Mbc3;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
static ROM: &[u8] = include_bytes!("../../dmg-lib/roms/pht-pz.gbc");

const MODE: Mode = Mode::CGB;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

/// WebAssembly-enabled emulator.
#[wasm_bindgen]
pub struct Dmg(dmg_lib::Dmg<Mbc, CanvasVideoOutput, ()>);

#[wasm_bindgen]
pub fn init_log() {
    console_log::init_with_level(log::Level::Debug).expect("Error initializing log");
}

#[wasm_bindgen]
impl Dmg {
    pub fn with_video(video: CanvasVideoOutput) -> Self {
        let dmg = Builder::default()
            .with_mode(MODE)
            .with_palette(PALETTE)
            .with_video(video)
            .with_cartridge(Mbc::from_bytes(ROM))
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
        "KeyZ" => Some(Btn(A)),
        "KeyX" => Some(Btn(B)),
        "Enter" => Some(Btn(Start)),
        "ShiftRight" => Some(Btn(Select)),
        "ArrowLeft" => Some(Dir(Left)),
        "ArrowRight" => Some(Dir(Right)),
        "ArrowUp" => Some(Dir(Up)),
        "ArrowDown" => Some(Dir(Down)),
        _ => None,
    }
}
