use dmg_driver_rodio::apu::RodioSamples;
use dmg_driver_sdl2::{apu::create_device, ppu::Sdl2VideoOutput};
use dmg_lib::{
    apu::device::Stereo44100,
    cartridge::{Cartridge, Mbc5},
    joypad::{Btn, Dir, Key},
    ppu::palette::{Palette, *},
    Builder, Dmg, Mode,
};
use dmg_peripheral_camera::PoketCamera;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    Sdl,
};
use std::{
    thread,
    time::{Duration, Instant},
};

static ROM: &[u8] =
    include_bytes!("../roms/Legend of Zelda, The - Link's Awakening DX (U) (V1.2) [C][!].gbc");

const RODIO: bool = true;
const SCALE: u32 = 2;
const MODE: Mode = Mode::GB;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

fn create_emulator(sdl: &Sdl) -> Dmg<impl Cartridge, Sdl2VideoOutput, Stereo44100<i16>> {
    let window = sdl
        .video()
        .unwrap()
        .window("DMG", 160 * SCALE, 144 * SCALE)
        .position_centered()
        .build()
        .expect("Error creating SDL window");

    let canvas = window
        .into_canvas()
        .build()
        .expect("Error creating SDL canvas");

    Builder::default()
        .with_mode(MODE)
        .with_palette(PALETTE)
        .with_video(Sdl2VideoOutput::from_canvas(canvas))
        .with_cartridge(())
        .with_cartridge(Mbc5::from_bytes(ROM))
        //.with_cartridge(PoketCamera::new(()))
        //.with_cartridge(())
        .with_audio()
        //.skip_boot()
        .build()
}

fn main() {
    //env_logger::init();

    let sdl = sdl2::init().unwrap();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut dmg = create_emulator(&sdl);

    let _device = if RODIO {
        eprintln!("Using Rodio for audio");
        let device = rodio::default_output_device().unwrap();
        let queue = rodio::Sink::new(&device);
        let source = RodioSamples::new(dmg.mmu().apu().samples());
        queue.append(source);
        queue.play();
        (Some(queue), None)
    } else {
        eprintln!("Using SDL for audio");
        let audio = sdl.audio().unwrap();
        let device = create_device(&audio, dmg.mmu().apu().samples()).unwrap();
        device.resume();
        (None, Some(device))
    };

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
        let sleep = if event_pump.keyboard_state().is_scancode_pressed(Scancode::S) {
            Duration::new(0, 1_000_000_000 / 30)
        } else {
            Duration::new(0, 1_000_000_000 / 60)
        };
        if time < sleep && !event_pump.keyboard_state().is_scancode_pressed(Scancode::F) {
            thread::sleep(sleep - time);
        }
    }
}
