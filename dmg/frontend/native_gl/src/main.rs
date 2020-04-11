use dmg_backend_gl::ppu::GLVideo;
use dmg_lib::{
    apu::device::Audio,
    cartridge::Cartridge,
    joypad::{Btn, Dir, Key},
    ppu::{palette::DMG, Video},
    Builder, Dmg, Mode,
};
use imgui::Context;
use imgui_opengl_renderer::Renderer;
use imgui_sdl2::ImguiSdl2;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    EventPump,
};
use std::{
    thread,
    time::{Duration, Instant},
};

static ROM: &[u8] = include_bytes!("../../native/roms/Tetris-USA.gb");

fn main() {
    let sdl = sdl2::init().expect("Error initializing SDL");
    let video = sdl.video().expect("Error initializing video");
    let window = video
        .window("DMG - GL", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .expect("Error creating window");

    let gl_ctx = window
        .gl_create_context()
        .expect("Error creating GL context");
    window
        .gl_make_current(&gl_ctx)
        .expect("Error setting GL context");

    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut imgui = Context::create();
    let imgui_gl = Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);
    let mut imgui_sdl = ImguiSdl2::new(&mut imgui, &window);

    let mut emulator = Builder::default()
        .with_mode(Mode::CGB)
        .with_cartridge(dmg_lib::cartridge::from_bytes(ROM).expect("Error creating cartridge"))
        .with_video(GLVideo::new())
        .build();
    emulator.mmu_mut().ppu_mut().pal_mut().set_color_pal(DMG);

    let mut event_pump = sdl.event_pump().expect("Error creating event pump");
    let mut carry = Duration::new(0, 0);
    loop {
        let time = Instant::now();

        if handle_input(&mut event_pump, &mut emulator, &mut imgui_sdl) {
            break;
        }

        emulator.emulate_frame();

        unsafe {
            gl::Viewport(0, 0, 800, 600);
            gl::ClearColor(0.5, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        imgui_sdl.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        imgui::Window::new(imgui::im_str!("Display"))
            .always_auto_resize(true)
            .resizable(false)
            .build(&ui, || {
                let texture = emulator.mmu().ppu().video().texture();
                let scale = 3.0;
                imgui::Image::new(
                    imgui::TextureId::from(texture as usize),
                    [160.0 * scale, 144.0 * scale],
                )
                .border_col([1.0; 4])
                .build(&ui);
            });
        imgui_sdl.prepare_render(&ui, &window);
        imgui_gl.render(ui);

        let elapsed = time.elapsed() + carry;
        let sleep = Duration::new(0, 1_000_000_000 / 60);
        if elapsed < sleep {
            carry = Duration::new(0, 0);
            thread::sleep(sleep - elapsed);
        } else {
            carry = elapsed - sleep;
        }

        window.gl_swap_window();
    }
}

fn handle_input(
    event_pump: &mut EventPump,
    dmg: &mut Dmg<impl Cartridge, impl Video, impl Audio>,
    imgui_sdl: &mut ImguiSdl2,
) -> bool {
    let joypad = dmg.mmu_mut().joypad_mut();
    for event in event_pump.poll_iter() {
        match event {
            Event::Window {
                win_event: WindowEvent::Close,
                ..
            } => return true,
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
