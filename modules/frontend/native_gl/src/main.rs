use dmg_backend_gl::ppu::{shader::lcd::Lcd, GLVideo};
use dmg_lib::{
    apu::device::Audio,
    cartridge,
    cartridge::Cartridge,
    joypad::{Btn, Dir, Key},
    ppu::{
        palette::{DMG, GRAYSCALE},
        reg::{TileDataAddr, TileMapAddr},
        Video,
    },
    Builder, GameBoy, Mode,
};
use gl::types::*;
use imgui::{Context, StyleVar};
use imgui_opengl_renderer::Renderer;
use imgui_sdl2::ImguiSdl2;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    EventPump,
};
use std::{
    ptr, thread,
    time::{Duration, Instant},
};

static ROM: &[u8] =
    include_bytes!("../../native/roms/Star Wars Episode I - Racer (USA, Europe).gbc");

const MODE: Mode = Mode::CGB;

struct Ppu {
    display: bool,
    palette: bool,
    tiles: bool,
    background: bool,
    window: bool,
    shader: bool,
}

struct Cpu {
    registers: bool,
}

fn main() {
    let mut ppu = Ppu {
        display: true,
        palette: true,
        tiles: false,
        background: false,
        window: false,
        shader: false,
    };

    let mut cpu = Cpu { registers: false };

    let sdl = sdl2::init().expect("Error initializing SDL");
    let video = sdl.video().expect("Error initializing video");
    let window = video
        .window("DMG - GL", 800, 600)
        .opengl()
        .position_centered()
        //.fullscreen()
        .build()
        .expect("Error creating window");

    let gl_ctx = window
        .gl_create_context()
        .expect("Error creating GL context");
    window
        .gl_make_current(&gl_ctx)
        .expect("Error setting GL context");

    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    // tile data & tile maps
    let mut tile_data = [0; 4];
    let mut tile_map = [0; 2];
    let mut texture_data = Vec::new();
    for tile_data in tile_data.iter_mut() {
        unsafe { *tile_data = create_texture(128, 128) };
    }
    for tile_map in tile_map.iter_mut() {
        unsafe { *tile_map = create_texture(256, 256) };
    }
    unsafe {
        assert_eq!(gl::NO_ERROR, gl::GetError());
    }

    let mut imgui = Context::create();
    let imgui_gl = Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);
    let mut imgui_sdl = ImguiSdl2::new(&mut imgui, &window);

    let mut emulator = Builder::default()
        .with_mode(MODE)
        .with_cartridge(cartridge::from_bytes(ROM).expect("Error creating cartridge"))
        .with_video(GLVideo::new(Lcd::default()))
        .build();
    emulator.mmu_mut().ppu_mut().pal_mut().set_color_pal(DMG);

    let mut event_pump = sdl.event_pump().expect("Error creating event pump");
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
        ui.main_menu_bar(|| {
            ui.menu(imgui::im_str!("cpu"), true, || {
                ui.checkbox(imgui::im_str!("Registers"), &mut cpu.registers);
            });
            ui.menu(imgui::im_str!("ppu"), true, || {
                ui.checkbox(imgui::im_str!("Display"), &mut ppu.display);
                ui.checkbox(imgui::im_str!("Palette"), &mut ppu.palette);
                ui.checkbox(imgui::im_str!("Tiles"), &mut ppu.tiles);
                ui.checkbox(imgui::im_str!("Background"), &mut ppu.background);
                ui.checkbox(imgui::im_str!("Window"), &mut ppu.window);
                ui.checkbox(imgui::im_str!("Shader"), &mut ppu.shader);
            });
        });
        if cpu.registers {
            #[rustfmt::skip]
            imgui::Window::new(imgui::im_str!("Registers"))
                .always_auto_resize(true)
                .resizable(false)
                .build(&ui, || {
                    imgui::InputInt::new(&ui, imgui::im_str!("AF"), &mut (emulator.cpu().reg().af() as _)).chars_hexadecimal(true).build();
                    imgui::InputInt::new(&ui, imgui::im_str!("BC"), &mut (emulator.cpu().reg().bc() as _)).chars_hexadecimal(true).build();
                    imgui::InputInt::new(&ui, imgui::im_str!("DE"), &mut (emulator.cpu().reg().de() as _)).chars_hexadecimal(true).build();
                    imgui::InputInt::new(&ui, imgui::im_str!("HL"), &mut (emulator.cpu().reg().hl() as _)).chars_hexadecimal(true).build();
                    imgui::InputInt::new(&ui, imgui::im_str!("PC"), &mut (emulator.cpu().reg().pc as _)).chars_hexadecimal(true).build();
                    imgui::InputInt::new(&ui, imgui::im_str!("SP"), &mut (emulator.cpu().reg().sp as _)).chars_hexadecimal(true).build();
                });
        }
        if ppu.display {
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
        }
        if ppu.shader {
            imgui::Window::new(imgui::im_str!("Shader"))
                .always_auto_resize(true)
                .resizable(false)
                .build(&ui, || {
                    let shader = emulator.mmu_mut().ppu_mut().video_mut().shader_mut();
                    ui.checkbox(imgui::im_str!("RGB"), &mut shader.rgb);
                    ui.checkbox(imgui::im_str!("Scanlines"), &mut shader.scanlines);
                    ui.checkbox(imgui::im_str!("Grayscale"), &mut shader.grayscale);
                });
        }
        if ppu.background {
            #[rustfmt::skip]
                imgui::Window::new(imgui::im_str!("Background"))
                .always_auto_resize(true)
                .build(&ui, || {
                    texture_data.clear();
                    let map = emulator.mmu().ppu().lcdc_stat().bg_tile_map();
                    let data = emulator.mmu().ppu().lcdc_stat().bg_win_tile_data();
                    emulator.mmu().ppu().tile_map(map, data, &mut texture_data);
                    assert_eq!(256 * 256, texture_data.len());
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, tile_map[0]);
                        #[rustfmt::skip]
                            gl::TexSubImage2D(
                            gl::TEXTURE_2D, 0, 0, 0, 256, 256, gl::RGB, gl::UNSIGNED_BYTE, texture_data.as_ptr() as _);
                        gl::BindTexture(gl::TEXTURE_2D, 0);
                    }
                    let texture = imgui::TextureId::from(tile_map[0] as usize);
                    imgui::Image::new(texture, [256.0, 256.0]).border_col([1.0; 4]).build(&ui);
                });
        }
        if ppu.window {
            #[rustfmt::skip]
                imgui::Window::new(imgui::im_str!("Window"))
                .always_auto_resize(true)
                .build(&ui, || {
                    let win = emulator.mmu().ppu().win();
                    imgui::InputInt::new(&ui, imgui::im_str!("WX"), &mut (win.wx as _)).read_only(true).build();
                    imgui::InputInt::new(&ui, imgui::im_str!("WY"), &mut (win.wy as _)).read_only(true).build();
                    texture_data.clear();
                    let map = emulator.mmu().ppu().lcdc_stat().win_tile_map();
                    let data = emulator.mmu().ppu().lcdc_stat().bg_win_tile_data();
                    emulator.mmu().ppu().tile_map(map, data, &mut texture_data);
                    assert_eq!(256 * 256, texture_data.len());
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, tile_map[1]);
                        #[rustfmt::skip]
                        gl::TexSubImage2D(
                            gl::TEXTURE_2D, 0, 0, 0, 256, 256, gl::RGB, gl::UNSIGNED_BYTE, texture_data.as_ptr() as _);
                        gl::BindTexture(gl::TEXTURE_2D, 0);
                    }
                    let texture = imgui::TextureId::from(tile_map[1] as usize);
                    imgui::Image::new(texture, [256.0, 256.0]).border_col([1.0; 4]).build(&ui);
                });
        }
        if ppu.tiles {
            #[rustfmt::skip]
                imgui::Window::new(imgui::im_str!("Tiles"))
                .always_auto_resize(true)
                .build(&ui, || {
                    let mut t = 0;
                    let banks = match MODE {
                        Mode::GB => 1,
                        Mode::CGB => 2,
                    };
                    for bank in 0..banks {
                        for (i, (addr, bank)) in [(TileDataAddr::X8000, bank), (TileDataAddr::X8800, bank)].iter().copied().enumerate() {
                            texture_data.clear();
                            emulator.mmu().ppu().tile_data(addr, bank, &mut texture_data);
                            assert_eq!(128 * 128, texture_data.len());
                            unsafe {
                                gl::BindTexture(gl::TEXTURE_2D, tile_data[t]);
                                #[rustfmt::skip]
                                    gl::TexSubImage2D(
                                    gl::TEXTURE_2D, 0, 0, 0, 128, 128, gl::RGB, gl::UNSIGNED_BYTE, texture_data.as_ptr() as _);
                                gl::BindTexture(gl::TEXTURE_2D, 0);
                            }
                            let [x, y] = ui.cursor_pos();
                            imgui::Image::new(imgui::TextureId::from(tile_data[t] as usize), [128.0, 128.0]).border_col([1.0; 4]).build(&ui);
                            if i == 0 {
                                ui.set_cursor_pos([x + 128.0 + 6.0, y]);
                            }
                            t += 1;
                        }
                    }
                });
        }
        if ppu.palette {
            #[rustfmt::skip]
                imgui::Window::new(imgui::im_str!("Color Palette"))
                .always_auto_resize(true)
                .resizable(false)
                .build(&ui, || {
                    #[rustfmt::skip]
                    let (bgp, obp) = match MODE {
                        Mode::GB => {
                            let pal = emulator.mmu().ppu().pal();
                            (vec![pal.bg_pal()],
                             vec![pal.ob_pal(0), pal.ob_pal(1)])
                        }
                        Mode::CGB => {
                            let pal = emulator.mmu().ppu().color_pal();
                            (vec![pal.bg_pal(0), pal.bg_pal(1), pal.bg_pal(2), pal.bg_pal(3), pal.bg_pal(4), pal.bg_pal(5), pal.bg_pal(6), pal.bg_pal(7)],
                             vec![pal.ob_pal(0), pal.ob_pal(1), pal.ob_pal(2), pal.ob_pal(3), pal.ob_pal(4), pal.ob_pal(5), pal.ob_pal(6), pal.ob_pal(7)])
                        }
                    };
                    ui.text("Background");
                    for (i, [pal0, pal1, pal2, pal3]) in bgp.iter().enumerate() {
                        let pal0 = [pal0[0] as f32 / 255.0, pal0[1] as f32 / 255.0, pal0[2] as f32 / 255.0, 1.0];
                        let pal1 = [pal1[0] as f32 / 255.0, pal1[1] as f32 / 255.0, pal1[2] as f32 / 255.0, 1.0];
                        let pal2 = [pal2[0] as f32 / 255.0, pal2[1] as f32 / 255.0, pal2[2] as f32 / 255.0, 1.0];
                        let pal3 = [pal3[0] as f32 / 255.0, pal3[1] as f32 / 255.0, pal3[2] as f32 / 255.0, 1.0];
                        let [x, y] = ui.cursor_pos();
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 0), pal0).build(&ui);
                        ui.set_cursor_pos([x + 24.0, y]);
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 1), pal1).build(&ui);
                        ui.set_cursor_pos([x + 24.0 * 2.0, y]);
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 2), pal2).build(&ui);
                        ui.set_cursor_pos([x + 24.0 * 3.0, y]);
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 3), pal3).build(&ui);
                    }
                    ui.text("Object");
                    for (i, [pal0, pal1, pal2, pal3]) in obp.iter().enumerate() {
                        let pal0 = [pal0[0] as f32 / 255.0, pal0[1] as f32 / 255.0, pal0[2] as f32 / 255.0, 1.0];
                        let pal1 = [pal1[0] as f32 / 255.0, pal1[1] as f32 / 255.0, pal1[2] as f32 / 255.0, 1.0];
                        let pal2 = [pal2[0] as f32 / 255.0, pal2[1] as f32 / 255.0, pal2[2] as f32 / 255.0, 1.0];
                        let pal3 = [pal3[0] as f32 / 255.0, pal3[1] as f32 / 255.0, pal3[2] as f32 / 255.0, 1.0];
                        let [x, y] = ui.cursor_pos();
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 0), pal0).build(&ui);
                        ui.set_cursor_pos([x + 24.0, y]);
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 1), pal1).build(&ui);
                        ui.set_cursor_pos([x + 24.0 * 2.0, y]);
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 2), pal2).build(&ui);
                        ui.set_cursor_pos([x + 24.0 * 3.0, y]);
                        imgui::ColorButton::new(&imgui::im_str!("pal#{}#{}", i, 3), pal3).build(&ui);
                    }
                });
        }

        imgui_sdl.prepare_render(&ui, &window);
        imgui_gl.render(ui);

        let elapsed = time.elapsed();
        let sleep = Duration::new(0, 1_000_000_000 / 60);
        if elapsed < sleep {
            thread::sleep(sleep - elapsed);
        }

        window.gl_swap_window();
    }
}

fn handle_input(
    event_pump: &mut EventPump,
    dmg: &mut GameBoy<impl Cartridge, impl Video, impl Audio>,
    imgui_sdl: &mut ImguiSdl2,
) -> bool {
    let joypad = dmg.mmu_mut().joypad_mut();
    for event in event_pump.poll_iter() {
        if !imgui_sdl.ignore_event(&event) {
            match event {
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
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

unsafe fn create_texture(width: GLsizei, height: GLsizei) -> GLuint {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
    #[rustfmt::skip]
    gl::TexImage2D(
        gl::TEXTURE_2D, 0, gl::RGB8 as _, width, height, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
    gl::BindTexture(gl::TEXTURE_2D, 0);
    texture
}
