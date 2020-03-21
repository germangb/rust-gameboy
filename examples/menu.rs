use crate::App;
use dmg::{
    cartridge::{Mbc1, Mbc3, RomAndRam},
    Dmg, Mode,
};
use imgui::{im_str, Ui};
use std::{
    io,
    path::{Path, PathBuf},
};

pub fn draw(ui: &Ui, app: &mut App) {
    ui.main_menu_bar(|| {
        ui.menu(im_str!("App"), true, || {
            ui.checkbox(im_str!("Skip boot"), &mut app.boot);
            ui.checkbox(im_str!("CGB (if available)"), &mut app.cgb);
        });

        let dirs = std::fs::read_dir(&app.roms_dir);
        ui.menu(im_str!("Library"), dirs.is_ok(), || {
            if ui.small_button(im_str!("Reload")) {
                match find_roms(&app.roms_dir, &mut app.roms_entries) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Error scanning ROMs = {}", err),
                }
            }
            ui.input_text(im_str!("Filter"), &mut app.roms_filter)
                .build();
            let im_entries: Vec<_> = app
                .roms_entries
                .iter()
                .map(|e| im_str!("{}", e.display()))
                .filter(|e| {
                    let filter = app.roms_filter.to_str().to_lowercase();
                    e.to_str().to_lowercase().contains(&filter)
                })
                .collect();
            let im_entries: Vec<_> = im_entries.iter().collect();
            if ui.list_box(im_str!("Roms"), &mut app.roms_selected, &im_entries[..], 24) {
                let rom = std::fs::read(&app.roms_entries[app.roms_selected as usize]).unwrap();
                let mut dmg = load_rom(&rom[..], app.cgb);
                dmg.mmu_mut().ppu_mut().set_palette(app.pal);
                if app.boot {
                    dmg.boot();
                }
                app.dmg = Some(dmg);
            }
        });

        ui.menu(im_str!("Palette"), true, || {
            for (i, pal) in dmg::ppu::palette::palettes().enumerate() {
                if ui.small_button(&im_str!("#{}", i+1)) {
                    app.pal = pal;
                    if let Some(dmg) = &mut app.dmg {
                        dmg.mmu_mut().ppu_mut().set_palette(pal);
                    }
                }
            }
        });
    });
}

fn find_roms(path: &Path, roms: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let is_dir = entry.file_type()?.is_dir();
        if is_dir {
            eprintln!("scanning dir = {}", path.display());
            match find_roms(&path, roms) {
                Ok(_) => {}
                Err(err) => eprintln!("Error finding toms in {} : {}", path.display(), err),
            }
        } else {
            match path.extension() {
                Some(ext) if ext == "gb" => {
                    eprintln!("found rom = {}", path.display());
                    roms.push(path);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn load_rom(rom: &[u8], cgb: bool) -> Dmg {
    let mode = match rom[0x143] {
        0x80 if cgb => Mode::CGB,
        0xc0 => Mode::CGB,
        _ => Mode::GB,
    };
    match rom[0x147] {
        0x00 | 0x08 | 0x09 => Dmg::new(RomAndRam::from_bytes(rom), mode),
        0x01 | 0x02 | 0x03 => Dmg::new(Mbc1::from_bytes(rom), mode),
        0x0f | 0x10 | 0x11 | 0x12 | 0x13 => Dmg::new(Mbc3::from_bytes(rom), mode),
        _ => Dmg::new(Mbc3::from_bytes(rom), mode),
    }
}
