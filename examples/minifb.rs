use minifb::{Scale, Window, WindowOptions};
use std::{thread, time::Duration};

fn main() {
    let mut opt = WindowOptions::default();
    opt.scale = Scale::X2;

    let mut window = Window::new("Window", 160, 144, opt).unwrap();
    let mut buffer = vec![0; 160 * 144];

    while window.is_open() {
        window.update_with_buffer(&buffer, 160, 144).unwrap();
        thread::sleep(Duration::new(0, 1_000_000_000 / 60));
    }
}
