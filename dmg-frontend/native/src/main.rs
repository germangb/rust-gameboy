use std::{
    thread,
    time::{Duration, Instant},
};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    Sdl,
};

use dmg_driver_rodio::apu::RodioAudioOutput;
use dmg_driver_sdl2::{apu::Sdl2AudioOutput, ppu::Sdl2VideoOutput};
use dmg_lib::{
    apu::AudioOutput,
    cartridge::{Cartridge, Mbc3, Mbc5},
    joypad::{Btn, Dir, Key},
    ppu::palette::{Palette, *},
    Builder, Dmg, Mode,
};
use dmg_peripheral_camera::PoketCamera;
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioStatus},
    render::TextureAccess::Target,
};
use std::sync::mpsc::{Receiver, Sender};

static ROM: &[u8] =
    include_bytes!("../roms/Tetris-USA.gb");

const SCALE: u32 = 2;
const MODE: Mode = Mode::GB;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

fn emulator(sdl: Sdl) -> Dmg<impl Cartridge, Sdl2VideoOutput, impl AudioOutput> {
    let video = sdl.video().unwrap();
    let window = video
        .window("DMG", 160 * SCALE, 144 * SCALE)
        .position_centered()
        .build()
        .expect("Error creating SDL window");

    let canvas = window
        .into_canvas()
        .build()
        .expect("Error creating SDL canvas");
    let video = Sdl2VideoOutput::from_canvas(canvas);

    let cartridge = Mbc3::from_bytes(ROM);
    //let cartridge = PoketCamera::with_sensor(());

    let audio = sdl.audio().unwrap();

    let mut dmg = Builder::default()
        .with_mode(MODE)
        .with_palette(PALETTE)
        .with_video(video)
        .with_audio(Sdl2AudioOutput::new(&audio).unwrap())
        .with_cartridge(cartridge)
        .build();

    dmg
}

fn main() {
    //env_logger::init();

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
