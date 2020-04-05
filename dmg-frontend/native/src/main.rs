use dmg_driver_rodio::apu::DmgSource;
use dmg_driver_sdl2::ppu::SdlVideo;
use dmg_lib::{
    apu::device::Stereo44100,
    cartridge::Mbc5,
    joypad::{Btn, Dir, Joypad, Key},
    ppu::palette::{Palette, *},
    Builder, Mode,
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

const WINDOW_SCALE: u32 = 2;
const PALETTE: Palette = NINTENDO_GAMEBOY_BLACK_ZERO;

static ROM: &[u8] = include_bytes!("../roms/Tetris-USA.gb");
static TEST: &[u8] = include_bytes!("../../../../gb-test-roms/cpu_instrs/cpu_instrs.gb");

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
        .with_cartridge(Mbc5::new(TEST))
        .with_audio::<Stereo44100<i16>>()
        .with_palette(PALETTE)
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

        // handle input
        if handle_input(&mut pump, emulator.mmu_mut().joypad_mut()) {
            break;
        }

        // emulate (1/60) seconds-worth of clock cycles
        emulator.emulate_frame();
        emulator.mmu_mut().ppu_mut().video_mut().present();

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

fn handle_input(pump: &mut EventPump, joypad: &mut Joypad) -> bool {
    for event in pump.poll_iter() {
        match event {
            Event::Window {
                win_event: WindowEvent::Close,
                ..
            } => return true,
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(key) = map_scancode(scancode) {
                    joypad.press(key)
                }
            }
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(key) = map_scancode(scancode) {
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
