use dmg_lib::{
    cartridge::{Mbc1, Mbc3, Mbc5, ZeroRom},
    joypad::{Btn, Dir, Key},
    ppu::palette::{Palette, NINTENDO_GAMEBOY_BLACK_ZERO},
    Dmg, Mode,
};
use dmg_sdl2::{audio::Sdl2AudioOutput, video::Sdl2VideoOutput};
use log::info;
use sdl2::{
    audio::AudioSpecDesired,
    event::{Event, WindowEvent},
    keyboard::Scancode,
    video::FullscreenType,
};
use std::{
    thread,
    time::{Duration, Instant},
};

const SCALE: u32 = 4;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

fn main() {
    env_logger::init();

    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let video = sdl.video().unwrap();
    let mut window = video
        .window("DMG - SDL2", 160 * SCALE, 144 * SCALE)
        .position_centered()
        .resizable()
        .build()
        .expect("Error creating SDL window");
    let canvas = window
        .into_canvas()
        .build()
        .expect("Error creating SDL canvas");

    let audio = sdl.audio().unwrap();

    let rom = include_bytes!("../../dmg_lib/roms/Cannon Fodder (Europe) (En,Fr,De,Es,It).gbc");
    let mut dmg = Dmg::new(
        Mbc5::from_bytes(&rom[..]),
        Mode::CGB,
        Sdl2VideoOutput::from_canvas(canvas),
        Sdl2AudioOutput::new(&audio).unwrap(),
    );
    dmg.mmu_mut().ppu_mut().set_palette(PALETTE);
    dmg.boot();

    'mainLoop: loop {
        let time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => break 'mainLoop,
                Event::KeyDown {
                    scancode: Some(key),
                    ..
                } => match key {
                    Scancode::Z => dmg.mmu_mut().joypad_mut().press(Key::Btn(Btn::A)),
                    Scancode::X => dmg.mmu_mut().joypad_mut().press(Key::Btn(Btn::B)),
                    Scancode::RShift => dmg.mmu_mut().joypad_mut().press(Key::Btn(Btn::Select)),
                    Scancode::Return => dmg.mmu_mut().joypad_mut().press(Key::Btn(Btn::Start)),
                    Scancode::Left => dmg.mmu_mut().joypad_mut().press(Key::Dir(Dir::Left)),
                    Scancode::Right => dmg.mmu_mut().joypad_mut().press(Key::Dir(Dir::Right)),
                    Scancode::Up => dmg.mmu_mut().joypad_mut().press(Key::Dir(Dir::Up)),
                    Scancode::Down => dmg.mmu_mut().joypad_mut().press(Key::Dir(Dir::Down)),
                    Scancode::R => {
                        info!("{:#?}", dmg.cpu());
                    }
                    _ => {}
                },
                Event::KeyUp {
                    scancode: Some(key),
                    ..
                } => match key {
                    Scancode::Z => dmg.mmu_mut().joypad_mut().release(Key::Btn(Btn::A)),
                    Scancode::X => dmg.mmu_mut().joypad_mut().release(Key::Btn(Btn::B)),
                    Scancode::RShift => dmg.mmu_mut().joypad_mut().release(Key::Btn(Btn::Select)),
                    Scancode::Return => dmg.mmu_mut().joypad_mut().release(Key::Btn(Btn::Start)),
                    Scancode::Left => dmg.mmu_mut().joypad_mut().release(Key::Dir(Dir::Left)),
                    Scancode::Right => dmg.mmu_mut().joypad_mut().release(Key::Dir(Dir::Right)),
                    Scancode::Up => dmg.mmu_mut().joypad_mut().release(Key::Dir(Dir::Up)),
                    Scancode::Down => dmg.mmu_mut().joypad_mut().release(Key::Dir(Dir::Down)),
                    _ => {}
                },
                _ => {}
            }
        }

        dmg.emulate_frame();
        dmg.mmu_mut().ppu_mut().video_output_mut().present();

        let time = time.elapsed();
        let sleep = Duration::new(0, 1_000_000_000 / 60);
        if time < sleep {
            thread::sleep(sleep - time);
        }
    }
}
