#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
#![deny(unused_variables)]
#![deny(unused_mut)]
#![deny(unused_imports)]
#![warn(clippy::style)]
#![deny(clippy::correctness)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
use dmg::{
    joypad::{Btn::*, Dir::*, Key::*},
    ppu::palette::{Palette, NINTENDO_GAMEBOY_BLACK_ZERO},
    Dmg,
};
use imgui::ImString;
use sdl2::keyboard::Scancode;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

mod cpu;
mod menu;
mod mmu;
mod ppu;

pub struct App {
    dmg: Option<Dmg>,
    texture: gl::types::GLuint,
    mem: Vec<u16>,
    mem_input: u16,
    boot: bool,
    cgb: bool,
    pal: Palette,
    roms_dir: PathBuf,
    roms_filter: ImString,
    roms_selected: i32,
    roms_entries: Vec<PathBuf>,
}

fn main() {
    let window = imgui_very_quick::builder()
        .background(0.5, 0.5, 0.5, 1.0)
        .build()
        .unwrap();

    let mut app = App {
        dmg: None,
        texture: 0,
        mem: Vec::new(),
        mem_input: 0,
        boot: true,
        cgb: true,
        pal: NINTENDO_GAMEBOY_BLACK_ZERO,
        roms_dir: PathBuf::from("roms"),
        roms_filter: ImString::with_capacity(1024),
        roms_selected: 0,
        roms_entries: Vec::new(),
    };

    gl::load_with(|s| window.gl_get_proc_addr(s) as _);

    ppu::init(&mut app);
    cpu::init(&mut app);
    mmu::init(&mut app);

    window
        .run(|ui, e| {
            let time = Instant::now();

            let joy = &[
                Dir(Down),
                Dir(Up),
                Dir(Left),
                Dir(Right),
                Btn(Start),
                Btn(Select),
                Btn(B),
                Btn(A),
            ];
            let key = &[
                Scancode::Down,
                Scancode::Up,
                Scancode::Left,
                Scancode::Right,
                Scancode::Return,
                Scancode::RShift,
                Scancode::X,
                Scancode::Z,
            ];

            if let Some(dmg) = &mut app.dmg {
                for (j, k) in joy.iter().zip(key) {
                    if e.keyboard_state().is_scancode_pressed(*k) {
                        dmg.mmu_mut().joypad_mut().press(*j);
                    } else {
                        dmg.mmu_mut().joypad_mut().release(*j);
                    }
                }

                dmg.emulate_frame();
            }

            menu::draw(ui, &mut app);
            ppu::draw(ui, &mut app);
            cpu::draw(ui, &mut app);
            mmu::draw(ui, &mut app);

            let elapsed = time.elapsed();
            let sleep = Duration::new(0, 1_000_000_000 / 60);
            if elapsed < sleep {
                std::thread::sleep(sleep - elapsed);
            }
            Ok(())
        })
        .unwrap();
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &mut self.texture);
        }
    }
}
