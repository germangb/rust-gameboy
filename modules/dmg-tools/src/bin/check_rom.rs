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
use colored::Colorize;
use std::{env, fs::File, io::Read};

enum TestResult {
    Ok,
    Err(String),
    Ignore,
}

impl TestResult {
    fn is_err(&self) -> bool {
        matches!(self, TestResult::Err(_))
    }
}

struct Test {
    title: Vec<u8>,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    licensee_code: [u8; 2],

    logo: TestResult,
    header_checksum: TestResult,
    global_checksum: TestResult,
}

impl Test {
    fn into_result(self) -> Result<Self, Self> {
        if self.logo.is_err() || self.header_checksum.is_err() || self.global_checksum.is_err() {
            Err(self)
        } else {
            Ok(self)
        }
    }
}

// Check the nintendo logo located at [0x0104..=0x133] in the ROM header. Every
// cartridge must contain this information. If it's not there, the program won't
// dmg-data.
fn check_logo(rom: &[u8]) -> TestResult {
    static LOGO: &[u8] = &[0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83,
                           0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e,
                           0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63,
                           0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e];

    let logo = &rom[0x104..=0x133];
    let fail = LOGO.iter().zip(logo).enumerate().find(|(_, (a, b))| a != b);

    if let Some((i, (a, b))) = fail {
        TestResult::Err(format!("The logo comparison failed at the {}th byte ({:02x} != {:02x})",
                                i, a, b))
    } else {
        TestResult::Ok
    }
}

// Computes the header checksum and compares its value to the reference stored
// at 0x014d in the rom header. If the values don't match, the rom won't
// dmg-data.
fn check_header_checksum(bytes: &[u8]) -> TestResult {
    let x = bytes[0x134..=0x14c].iter()
                                .fold(0u8, |x, b| x.wrapping_sub(*b).wrapping_sub(1));
    let sum = bytes[0x14d];
    if x == sum {
        TestResult::Ok
    } else {
        TestResult::Err(format!(
            "The computed header sum ({:02x}) don't match the one in the cartridge ({:#02x})",
            x, sum
        ))
    }
}

// Not checked in the gamebot checksum.
fn check_global_checksum(_bytes: &[u8]) -> TestResult {
    TestResult::Ignore
}

fn check_rom(rom: &[u8]) -> Result<Test, Test> {
    // Title of the game in UPPER CASE ASCII. If it is less than 16 characters then
    // the remaining bytes are filled with 00's. When inventing the CGB, Nintendo
    // has reduced the length of this area to 15 characters, and some months later
    // they had the fantastic idea to reduce it to 11 characters only. The new
    // meaning of the ex-title bytes is described below.
    const TITLE_LEN: usize = 11;

    Test { title: rom.iter().skip(0x134).take(TITLE_LEN).copied().collect(),
           cartridge_type: rom[0x147],
           rom_size: rom[0x148],
           ram_size: rom[0x149],
           licensee_code: [rom[0x144], rom[0x145]],
           logo: check_logo(rom),
           header_checksum: check_header_checksum(rom),
           global_checksum: check_global_checksum(rom) }.into_result()
}

fn display(test: Test) {
    let cartridge_type = test.cartridge_type;
    #[rustfmt::skip]
    let cartridge_name = vec![
        (0x00, "ROM ONLY"),               (0x19, "MBC5"),
        (0x01, "MBC1"),                   (0x1A, "MBC5+RAM"),
        (0x02, "MBC1+RAM"),               (0x1B, "MBC5+RAM+BATTERY"),
        (0x03, "MBC1+RAM+BATTERY"),       (0x1C, "MBC5+RUMBLE"),
        (0x05, "MBC2"),                   (0x1D, "MBC5+RUMBLE+RAM"),
        (0x06, "MBC2+BATTERY"),           (0x1E, "MBC5+RUMBLE+RAM+BATTERY"),
        (0x08, "ROM+RAM"),                (0x20, "MBC6"),
        (0x09, "ROM+RAM+BATTERY"),        (0x22, "MBC7+SENSOR+RUMBLE+RAM+BATTERY"),
        (0x0B, "MMM01"),
        (0x0C, "MMM01+RAM"),
        (0x0D, "MMM01+RAM+BATTERY"),
        (0x0F, "MBC3+TIMER+BATTERY"),
        (0x10, "MBC3+TIMER+RAM+BATTERY"), (0xFC, "POCKET CAMERA"),
        (0x11, "MBC3"),                   (0xFD, "BANDAI TAMA5"),
        (0x12, "MBC3+RAM"),               (0xFE, "HuC3"),
        (0x13, "MBC3+RAM+BATTERY"),       (0xFF, "HuC1+RAM+BATTERY"),
    ]
        .into_iter()
        .find(|(b, _)| *b == cartridge_type)
        .map(|(_, name)| format!("({})", name))
        .unwrap_or_else(|| "".to_string());

    let title_ascii = test.title
                          .into_iter()
                          .take_while(|b| b.is_ascii())
                          .collect();
    let title = String::from_utf8(title_ascii).expect("Error parsing title as UTF8");

    let licensee_code = test.licensee_code;
    #[rustfmt::skip]
    let licensee_name = vec![
    // (&[b'0', b'0'], "none"),
       (&[0x00, 0x00], "none"),               (&[b'0', b'1'], "Nintendo R&D1"), (&[b'0', b'8'], "Capcom"),
       (&[b'1', b'3'], "Electronic Arts"),    (&[b'1', b'8'], "Hudson Soft"),   (&[b'1', b'9'], "b-ai"),
       (&[b'2', b'0'], "kss"),                (&[b'2', b'2'], "pow"),           (&[b'2', b'4'], "PCM Complete"),
       (&[b'2', b'5'], "san-x"),              (&[b'2', b'8'], "Kemco Japan"),   (&[b'2', b'9'], "seta"),
       (&[b'3', b'0'], "Viacom"),             (&[b'3', b'1'], "Nintendo"),      (&[b'3', b'2'], "Bandai"),
       (&[b'3', b'3'], "Ocean/Acclaim"),      (&[b'3', b'4'], "Konami"),        (&[b'3', b'5'], "Hector"),
       (&[b'3', b'7'], "Taito"),              (&[b'3', b'8'], "Hudson"),        (&[b'3', b'9'], "Banpresto"),
       (&[b'4', b'1'], "Ubi Soft"),           (&[b'4', b'2'], "Atlus"),         (&[b'4', b'4'], "Malibu"),
       (&[b'4', b'6'], "angel"),              (&[b'4', b'7'], "Bullet-Proof"),  (&[b'4', b'9'], "irem"),
       (&[b'5', b'0'], "Absolute"),           (&[b'5', b'1'], "Acclaim"),       (&[b'5', b'2'], "Activision"),
       (&[b'5', b'3'], "American sammy"),     (&[b'5', b'4'], "Konami"),        (&[b'5', b'5'], "Hi tech entertainment"),
       (&[b'5', b'6'], "LJN"),                (&[b'5', b'7'], "Matchbox"),      (&[b'5', b'8'], "Mattel"),
       (&[b'5', b'9'], "Milton Bradley"),     (&[b'6', b'0'], "Titus"),         (&[b'6', b'1'], "Virgin"),
       (&[b'6', b'4'], "LucasArts"),          (&[b'6', b'7'], "Ocean"),         (&[b'6', b'9'], "Electronic Arts"),
       (&[b'7', b'0'], "Infogrames"),         (&[b'7', b'1'], "Interplay"),     (&[b'7', b'2'], "Broderbund"),
       (&[b'7', b'3'], "sculptured"),         (&[b'7', b'5'], "sci"),           (&[b'7', b'8'], "THQ"),
       (&[b'7', b'9'], "Accolade"),           (&[b'8', b'0'], "misawa"),        (&[b'8', b'3'], "lozc"),
       (&[b'8', b'6'], "tokuma shoten i*"),   (&[b'8', b'7'], "tsukuda ori*"),  (&[b'9', b'1'], "Chunsoft"),
       (&[b'9', b'2'], "Video system"),       (&[b'9', b'3'], "Ocean/Acclaim"), (&[b'9', b'5'], "Varie"),
       (&[b'9', b'6'], "Yonezawa/s'pal"),     (&[b'9', b'7'], "Kaneko"),        (&[b'9', b'9'], "Pack in soft"),
       (&[b'A', b'4'], "Konami (Yu-Gi-Oh!)"),
    ]
        .into_iter()
        .find(|(code, _)| **code == licensee_code)
        .map(|(_, name)| format!("`{}`", name.trim()))
        .unwrap_or(format!("{:?}", licensee_code));

    let rom_size = test.rom_size;
    let rom_size_name = vec![(0x00, "32KByte (no ROM banking)"),
                             (0x01, "64KByte (4 banks)"),
                             (0x02, "128KByte (8 banks)"),
                             (0x03, "256KByte (16 banks)"),
                             (0x04, "512KByte (32 banks)"),
                             (0x05, "1MByte (64 banks)  - only 63 banks used by MBC1"),
                             (0x06, "2MByte (128 banks) - only 125 banks used by MBC1"),
                             (0x07, "4MByte (256 banks)"),
                             (0x08, "8MByte (512 banks)"),
                             (0x52, "1.1MByte (72 banks)"),
                             (0x53, "1.2MByte (80 banks)"),
                             (0x54, "1.5MByte (96 banks)"),].into_iter()
                                                            .find(|(rom, _)| *rom == rom_size)
                                                            .map(|(_, name)| format!("({})", name))
                                                            .unwrap_or_else(|| "".to_string());

    let ram_size = test.ram_size;
    let ram_size_name =
        vec![(0x00, "None"),
             (0x01, "2 KBytes"),
             (0x02, "8 Kbytes"),
             (0x03, "32 KBytes (4 banks of 8KBytes each)"),
             (0x04, "128 KBytes (16 banks of 8KBytes each)"),
             (0x05, "64 KBytes (8 banks of 8KBytes each)"),].into_iter()
                                                            .find(|(ram, _)| *ram == ram_size)
                                                            .map(|(_, name)| format!("({})", name))
                                                            .unwrap_or_else(|| "".to_string());

    eprintln!("Cartridge\n========================");
    eprintln!("Title .................. `{}`", title);
    eprintln!("Licensee ............... {}", licensee_name);
    eprintln!("Type ................... {:02X}h {}",
              test.cartridge_type, cartridge_name);
    eprintln!("ROM Size ............... {:02X}h {}",
              rom_size, rom_size_name);
    eprintln!("RAM Size ............... {:02X}h {}",
              ram_size, ram_size_name);
    eprintln!();

    let tests = &[("Logo ................... ", test.logo),
                  ("Header Checksum ........ ", test.header_checksum),
                  ("Global Checksum ........ ", test.global_checksum)];

    eprintln!("Tests\n========================");

    for (title, test) in tests {
        eprint!("{}", title);
        match test {
            TestResult::Ok => eprintln!("{}", "Ok".green()),
            TestResult::Err(e) => eprintln!("{}", format!("Error - {}", e).red()),
            TestResult::Ignore => eprintln!("{}", "Ignored".yellow()),
        }
    }
}

fn rom_path_from_args() -> Option<String> {
    let path = env::args().skip(1).fold(String::new(), |mut str, arg| {
                                      str.push_str(&arg);
                                      str.push(' ');
                                      str
                                  });
    let path = path.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

fn main() {
    let mut rom = Vec::new();

    eprintln!("File\n========================");
    if let Some(path) = rom_path_from_args() {
        eprintln!("ROM path ............... `{}`", path);

        let mut file = File::open(path).expect("Error opening file");
        file.read_to_end(&mut rom)
            .expect("Error reading file contents");
    } else {
        eprintln!("ROM path: `STDIN`");

        std::io::stdin().read_to_end(&mut rom)
                        .expect("Error reading STDIN contents");
    }

    eprintln!("File size .............. {}", rom.len());
    eprintln!();

    match check_rom(&rom) {
        Ok(test) => display(test),
        Err(test) => {
            display(test);
            std::process::exit(1)
        }
    }
}
