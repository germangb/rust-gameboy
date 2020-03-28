use dmg_lib::{
    apu::AudioOutput,
    cartridge::{Cartridge, Mbc1, Mbc3, Mbc5},
    joypad::{Btn, Dir, Key},
    ppu::palette::{Palette, *},
    Builder, Dmg, Mode,
};
use dmg_sdl2::{audio::Sdl2AudioOutput, video::Sdl2VideoOutput};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    Sdl,
};
use std::{
    thread,
    time::{Duration, Instant},
};

const MODE: Mode = Mode::CGB;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;
const SCALE: u32 = 2;

static ROM: &[u8] = include_bytes!("../../../dmg-lib/roms/pht-pz.gbca");

fn emulator(sdl: Sdl) -> Dmg<impl Cartridge, Sdl2VideoOutput, impl AudioOutput> {
    let video = sdl.video().unwrap();
    let canvas = video
        .window("DMG - SDL2", 160 * SCALE, 144 * SCALE)
        .position_centered()
        .resizable()
        .build()
        .expect("Error creating SDL window")
        .into_canvas()
        .build()
        .expect("Error creating SDL canvas");

    let audio = sdl.audio().expect("SDL audio error");

    Builder::default()
        .with_mode(MODE)
        .with_palette(PALETTE)
        .with_video(Sdl2VideoOutput::from_canvas(canvas))
        .with_audio(Sdl2AudioOutput::new(&audio).expect("SDL audio output error"))
        .with_cartridge(Mbc5::from_bytes(ROM))
        .build()
}

fn main() {
    env_logger::init();

    let sdl = sdl2::init().unwrap();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut dmg = emulator(sdl);

    'mainLoop: loop {
        let time = Instant::now();

        let joypad = dmg.mmu_mut().joypad_mut();

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
                    Scancode::Z => joypad.press(Key::Btn(Btn::A)),
                    Scancode::X => joypad.press(Key::Btn(Btn::B)),
                    Scancode::RShift => joypad.press(Key::Btn(Btn::Select)),
                    Scancode::Return => joypad.press(Key::Btn(Btn::Start)),
                    Scancode::Left => joypad.press(Key::Dir(Dir::Left)),
                    Scancode::Right => joypad.press(Key::Dir(Dir::Right)),
                    Scancode::Up => joypad.press(Key::Dir(Dir::Up)),
                    Scancode::Down => joypad.press(Key::Dir(Dir::Down)),
                    _ => {}
                },
                Event::KeyUp {
                    scancode: Some(key),
                    ..
                } => match key {
                    Scancode::Z => joypad.release(Key::Btn(Btn::A)),
                    Scancode::X => joypad.release(Key::Btn(Btn::B)),
                    Scancode::RShift => joypad.release(Key::Btn(Btn::Select)),
                    Scancode::Return => joypad.release(Key::Btn(Btn::Start)),
                    Scancode::Left => joypad.release(Key::Dir(Dir::Left)),
                    Scancode::Right => joypad.release(Key::Dir(Dir::Right)),
                    Scancode::Up => joypad.release(Key::Dir(Dir::Up)),
                    Scancode::Down => joypad.release(Key::Dir(Dir::Down)),
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
