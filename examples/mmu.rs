use crate::App;
use dmg::dev::Device;
use imgui::{im_str, Ui, Window};

pub fn init(app: &mut App) {
    app.mem.push(0xff42); // SCY
    app.mem.push(0xff43); // SCX
    app.mem.push(0xff4a); // WY
    app.mem.push(0xff4b); // WX
}

pub fn draw(ui: &Ui, app: &mut App) {
    if app.dmg.is_none() {
        return;
    }

    Window::new(im_str!("Memory"))
        .resizable(false)
        .always_auto_resize(true)
        .build(ui, || {
            let mut mem_input = app.mem_input as i32;
            ui.input_int(im_str!("Addr"), &mut mem_input)
                .chars_hexadecimal(true)
                .build();
            app.mem_input = (mem_input & 0xffff) as u16;
            if ui.small_button(im_str!("Add")) {
                app.mem_input &= 0xffff;
                app.mem.push(mem_input as u16);
                app.mem_input = 0;
            }
            ui.separator();

            if let Some(dmg) = &app.dmg {
                for addr in &app.mem {
                    let mut data = dmg.mmu().read(*addr) as i32;
                    ui.input_int(&im_str!("{:04x}", *addr), &mut data)
                        .read_only(true)
                        .chars_hexadecimal(true)
                        .build();
                }
            }
        });
}
