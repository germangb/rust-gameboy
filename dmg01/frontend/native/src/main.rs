use dmg_driver_rodio::apu::DmgSource;
use dmg_driver_sdl2::ppu::SdlVideo;
use dmg_lib::{
    apu::device::{Audio, Stereo44100},
    cartridge::{Cartridge, Mbc1, Mbc3, Mbc5},
    joypad::{Btn, Dir, Joypad, Key},
    ppu::{
        palette::{Palette, *},
        Video,
    },
    Builder, Dmg, Mode,
};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    EventPump,
};
use std::{
    thread,
    time::{Duration, Instant},
};

const WINDOW_SCALE: u32 = 4;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

// FIXME I think I broke the timer module (Mario 2 and 4)
static ROM: &[u8] = include_bytes!("../roms/Tennis (JUE) [!].gb");

fn main() {
    env_logger::init();

    let sdl = sdl2::init().unwrap();
    let canvas = sdl
        .video()
        .unwrap()
        .window("DMG", 160 * WINDOW_SCALE, 144 * WINDOW_SCALE)
        .position_centered()
        .build()
        .expect("Error creating SDL window")
        .into_canvas()
        .build()
        .expect("Error creating SDL canvas");

    let mut emulator = Builder::default()
        .with_video(SdlVideo::new(canvas))
        .with_cartridge(())
        .with_cartridge(Mbc3::new(ROM))
        .with_audio::<Stereo44100<i16>>()
        .with_palette(PALETTE)
        .with_mode(Mode::GB)
        .skip_boot()
        .build();

    // set up audio
    let device = rodio::default_output_device().expect("Error creating rodio device");
    let sink = rodio::Sink::new(&device);
    let source = DmgSource::new(emulator.mmu().apu());
    sink.append(source);
    sink.play();

    let mut pump = sdl.event_pump().unwrap();

    let mut carry = Duration::new(0, 0);

    loop {
        let time = Instant::now();

        if handle_input(&mut pump, &mut emulator) {
            break;
        }

        emulator.emulate_frame();
        emulator
            .mmu_mut()
            .ppu_mut()
            .video_mut()
            .canvas_mut()
            .present();

        let elapsed = time.elapsed() + carry;
        let sleep = Duration::new(0, 1_000_000_000 / 60);
        if elapsed < sleep {
            carry = Duration::new(0, 0);
            thread::sleep(sleep - elapsed);
        } else {
            carry = elapsed - sleep;
        }
    }
}

fn handle_input(
    pump: &mut EventPump,
    dmg: &mut Dmg<impl Cartridge, impl Video, impl Audio>,
) -> bool {
    let joypad = dmg.mmu_mut().joypad_mut();
    for event in pump.poll_iter() {
        match event {
            Event::Window {
                win_event: WindowEvent::Close,
                ..
            } => return true,
            Event::KeyDown {
                scancode: Some(Scancode::S),
                ..
            } => unimplemented!("screenshot"),
            Event::KeyDown {
                scancode: Some(s), ..
            } => {
                if let Some(key) = map_scancode(s) {
                    joypad.press(key)
                }
            }
            Event::KeyUp {
                scancode: Some(s), ..
            } => {
                if let Some(key) = map_scancode(s) {
                    joypad.release(key)
                }
            }
            _ => {}
        }
    }
    false
}

fn map_scancode(scancode: Scancode) -> Option<Key> {
    match scancode {
        Scancode::Z => Some(Key::Btn(Btn::A)),
        Scancode::X => Some(Key::Btn(Btn::B)),
        Scancode::RShift => Some(Key::Btn(Btn::Select)),
        Scancode::Return => Some(Key::Btn(Btn::Start)),
        Scancode::Left => Some(Key::Dir(Dir::Left)),
        Scancode::Right => Some(Key::Dir(Dir::Right)),
        Scancode::Up => Some(Key::Dir(Dir::Up)),
        Scancode::Down => Some(Key::Dir(Dir::Down)),
        _ => None,
    }
}
