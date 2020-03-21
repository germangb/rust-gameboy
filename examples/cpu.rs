use crate::App;
use imgui::{im_str, ImString, Ui, Window};

pub fn init(_app: &mut App) {}

pub fn draw(ui: &Ui, app: &mut App) {
    if let Some(dmg) = &app.dmg {
        Window::new(im_str!("CPU"))
            .resizable(false)
            .always_auto_resize(true)
            .build(ui, || {
                let cpu = dmg.cpu();
                let reg = cpu.reg();
                ui.checkbox(im_str!("HALT"), &mut cpu.halt());
                ui.checkbox(im_str!("IME"), &mut cpu.ime());
                ui.input_text(
                    im_str!("AF"),
                    &mut ImString::new(format!("{:02x} {:02x}", reg.a, reg.f)),
                )
                .read_only(true)
                .build();
                ui.input_text(
                    im_str!("BC"),
                    &mut ImString::new(format!("{:02x} {:02x}", reg.b, reg.c)),
                )
                .read_only(true)
                .build();
                ui.input_text(
                    im_str!("DE"),
                    &mut ImString::new(format!("{:02x} {:02x}", reg.d, reg.e)),
                )
                .read_only(true)
                .build();
                ui.input_text(
                    im_str!("HL"),
                    &mut ImString::new(format!("{:02x} {:02x}", reg.h, reg.l)),
                )
                .read_only(true)
                .build();
                ui.input_text(im_str!("SP"), &mut ImString::new(format!("{:04x}", reg.sp)))
                    .read_only(true)
                    .build();
                ui.input_text(im_str!("PC"), &mut ImString::new(format!("{:04x}", reg.pc)))
                    .read_only(true)
                    .build();
            });
    }
}
