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
use dmg_camera::{CameraSensor, PoketCamera, SENSOR_HEIGHT, SENSOR_WIDTH};
use dmg_driver_sdl2::{audio::Sdl2AudioOutput, video::Sdl2VideoOutput};
use dmg_lib::{
    apu::AudioOutput,
    cartridge::{Cartridge, Mbc3},
    joypad::{Btn, Dir, Key},
    ppu::palette::{Palette, *},
    Builder, Dmg, Mode,
};
use image::DynamicImage;
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

struct Sensor {
    image: image::GrayImage,
    offset: u32,
}

impl CameraSensor for Sensor {
    fn capture(&mut self, buf: &mut [[u8; SENSOR_WIDTH]; SENSOR_HEIGHT]) {
        for (i, row) in buf.iter_mut().enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                let i = (i as u32 + self.offset) % SENSOR_HEIGHT as u32;
                let j = (j as u32 + self.offset) % SENSOR_WIDTH as u32;
                *col = self.image.get_pixel(j, i)[0];
            }
        }
        self.offset = self.offset.wrapping_add(1);
    }
}

fn cartridge() -> impl Cartridge {
    static IMAGE: &[u8] = include_bytes!("../../rust.png");
    let image = image::load_from_memory(IMAGE)
        .map(DynamicImage::into_luma)
        .expect("Error loading image");
    PoketCamera::with_sensor(Sensor { image, offset: 0 });
    static ROM: &[u8] =
        include_bytes!("../../../../dmg-lib/roms/Pokemon - Yellow Version (UE) [C][!].gbc");

    Mbc3::from_bytes(ROM)
}

fn emulator(sdl: Sdl) -> Dmg<impl Cartridge, Sdl2VideoOutput, impl AudioOutput> {
    let video = sdl.video().unwrap();
    let canvas = video
        .window("DMG", 160 * SCALE, 144 * SCALE)
        .position_centered()
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
        .with_cartridge(cartridge())
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
