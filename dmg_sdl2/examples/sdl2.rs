use dmg_lib::{
    cartridge::{Mbc3, ZeroRom},
    joypad::{Btn, Dir, Key},
    ppu::palette::NINTENDO_GAMEBOY_BLACK_ZERO,
    Dmg, Mode,
};
use dmg_sdl2::{audio::Sdl2AudioOutput, video::Sdl2VideoOutput};
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

fn main() {
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

    let rom = include_bytes!("../../dmg_lib/roms/Tetris-USA.gb");
    let mut dmg = Dmg::new(
        Mbc3::from_bytes(&rom[..]),
        Mode::GB,
        Sdl2VideoOutput::from_canvas(canvas),
        Sdl2AudioOutput::new(&audio).unwrap(),
    );
    dmg.boot();

    'mainLoop: loop {
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
        let time = Instant::now();

        dmg.mmu_mut()
            .ppu_mut()
            .set_palette(NINTENDO_GAMEBOY_BLACK_ZERO);
        dmg.emulate_frame();
        dmg.mmu_mut().ppu_mut().video_output_mut().present();

        let time = time.elapsed();
        let sleep = Duration::new(0, 1_000_000_000 / 60);
        if time < sleep {
            thread::sleep(sleep - time);
        }
    }
}
