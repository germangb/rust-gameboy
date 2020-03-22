use crate::App;
use imgui::{im_str, Image, TextureId, Ui, Window};
use std::ptr;

pub fn init(app: &mut App) {
    unsafe {
        gl::GenTextures(1, &mut app.texture);
        gl::BindTexture(gl::TEXTURE_2D, app.texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB8 as _,
            160,
            144,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );
    }
}

pub fn draw(ui: &Ui, app: &mut App) {
    if let Some(dmg) = &app.dmg {
        unsafe {
            let buffer = dmg.mmu().ppu().buffer();
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                160,
                144,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                buffer.as_ptr() as _,
            );
        }

        Window::new(im_str!("Display"))
            .resizable(false)
            .always_auto_resize(true)
            .menu_bar(true)
            .build(ui, || {
                ui.menu_bar(|| {
                    ui.menu(im_str!("Scale"), true, || {
                        if ui.small_button(im_str!("x1")) {
                            app.display_scale = 1;
                        }
                        if ui.small_button(im_str!("x2")) {
                            app.display_scale = 2;
                        }
                        if ui.small_button(im_str!("x4")) {
                            app.display_scale = 4;
                        }
                    });
                });
                let texture = TextureId::from(app.texture as usize);
                let scale = app.display_scale as f32;
                Image::new(texture, [160.0 * scale, 144.0 * scale])
                    .border_col([1.0, 1.0, 1.0, 1.0])
                    .build(ui);
            });
    }
}
