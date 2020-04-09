use dmg_lib::{cartridge::Controller, map::Mapped};

pub static ROM: &[u8] = include_bytes!(env!("DMG_CAMERA_ROM"));

pub const SENSOR_W: usize = 128;
pub const SENSOR_H: usize = 112;

/// Trait to provide raw image data.
pub trait Sensor {
    fn capture(&mut self, buffer: &mut [[u8; SENSOR_W]; SENSOR_H]);
}

impl Sensor for () {
    fn capture(&mut self, _: &mut [[u8; 128]; 112]) {}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    Ram,
    Cam,
}

pub struct Cam {
    // The lower 3 bits of this register can be read and write. The other bits return '0'. Writing
    // any value with bit 0 set to '1' will start the capturing process. Any write with bit 0 set
    // to '0' is a normal write and won't trigger the capture. The value of bits 1 and 2 affects
    // the value written to registers 4, 5 and 6 of the M64282FP, which are used in 1-D filtering
    // mode (effects described in following chapters).
    //
    // Bit 0 of this register is also used to verify if the capturing process is finished. It
    // returns '1' when the hardware is working and '0' if the capturing process is over.
    //
    // When the capture process is active all RAM banks will return 00h when read (and writes are
    // ignored), but the register A000 can still be read to know when the transfer is finished.
    //
    // The capturing process can be stopped by writing a '0' to bit 0. When a '1' is written again
    // it will continue the previous capture process with the old capture parameters, even if the
    // registers are changed in between. If the process is stopped RAM can be read again.
    pub a000: u8,
    // This register is mapped to register 1 of M64282FP. It controls the output gain and the edge
    // operation mode.
    pub a001: u8,
    // This registers are mapped to registers 2 and 3 of M64282FP. They control the exposure time.
    // Register 2 is the MSB, register 3 is the LSB.
    //
    // u16 exposure_steps = [A003] | ([A002]<<8);
    pub a002: u8,
    pub a003: u8,
    // This register is mapped to register 7 of M64282FP. It sets the output voltage reference, the
    // edge enhancement ratio and it can invert the image.
    pub a004: u8,
    // This register is mapped to register 0 of M64282FP. It sets the output reference voltage and
    // enables the zero point calibration.
    pub a005: u8,
    // Those registers form a 4×4 matrix with 3 bytes per element. They handle dithering and
    // contrast, and they are sorted by rows:
    pub a006: [u8; 0x30],
}

pub struct PoketCamera<S: Sensor> {
    mode: Mode,
    sensor: S,
    buf: [[u8; SENSOR_W]; SENSOR_H],
    rom_bank: usize,
    ram: Vec<[u8; 0x2000]>,
    ram_bank: usize,
    ram_enabled: bool,
    cam: Cam,
}

impl<S: Sensor> PoketCamera<S> {
    pub fn new(sensor: S) -> Self {
        let mode = Mode::Ram;
        let buffer = [[0; SENSOR_W]; SENSOR_H];
        let rom_bank = 0;
        let ram = vec![[0; 0x2000]; 16];
        let ram_bank = 0;
        let ram_enabled = false;
        let cam = Cam {
            a000: 0,
            a001: 0,
            a002: 0,
            a003: 0,
            a004: 0,
            a005: 0,
            a006: [0; 0x30],
        };
        Self {
            mode,
            sensor,
            buf: buffer,
            rom_bank,
            ram,
            ram_bank,
            ram_enabled,
            cam,
        }
    }

    pub fn cam(&self) -> &Cam {
        &self.cam
    }

    pub fn cam_mut(&mut self) -> &mut Cam {
        &mut self.cam
    }

    fn capture(&mut self) {
        self.sensor.capture(&mut self.buf);

        for (i, row) in self.buf.iter_mut().enumerate() {
            for (j, pixel) in row.iter_mut().enumerate() {
                // invert
                if self.cam.a004 & 0x4 != 0 {
                    *pixel = !*pixel;
                }

                let tile_i = i / 8; // tile map index
                let tile_j = j / 8;

                let row = i % 8; // tile pixel index
                let col = 7 - (j % 8);

                const TILE_MAP_WIDTH: usize = SENSOR_W / 8;

                let tile_offset = 0x0100 + 16 * TILE_MAP_WIDTH * tile_i + 16 * tile_j;
                let tile_hi_offset = tile_offset + 2 * row;
                let tile_lo_offset = tile_hi_offset + 1;

                let mut hi = self.ram[0][tile_hi_offset];
                let mut lo = self.ram[0][tile_lo_offset];

                let col = col as u8;
                hi &= !(1 << col);
                lo &= !(1 << col);

                // dithering matrix
                let d_off = 12 * (i % 4) + 3 * (j % 4);
                let d_lo = self.cam.a006[d_off];
                let d_mi = self.cam.a006[d_off + 1];
                let d_hi = self.cam.a006[d_off + 2];

                if *pixel < d_lo {
                    lo |= 1 << col;
                    hi |= 1 << col;
                } else if *pixel < d_mi {
                    hi |= 1 << col;
                } else if *pixel < d_hi {
                    lo |= 1 << col;
                }

                // match *pixel {
                //     0x00..=0x39 => { /* white */ }
                //     0x3a..=0x79 => lo |= 1 << col,
                //     0x7a..=0xbf => hi |= 1 << col,
                //     0xc0..=0xff => {
                //         lo |= 1 << col;
                //         hi |= 1 << col;
                //     }
                // }

                self.ram[0][tile_hi_offset] = hi;
                self.ram[0][tile_lo_offset] = lo;
            }
        }
    }
}

impl<S: Sensor> Mapped for PoketCamera<S> {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => ROM[addr as usize],
            0x4000..=0x7fff => {
                let offset = 0x4000 * self.rom_bank;
                ROM[offset + addr as usize - 0x4000]
            }
            0xa000..=0xbfff => match self.mode {
                // Reading from RAM or registers is always enabled. Writing to registers is always
                // enabled. Disabled on reset.
                Mode::Ram => self.ram[self.ram_bank][addr as usize - 0xa000],
                Mode::Cam => match 0xa000 + (addr % 0x80) {
                    0xa000 => self.cam.a000 & 0x7,
                    0xa001 => 0,
                    0xa002 => 0,
                    0xa003 => 0,
                    0xa004 => 0,
                    0xa005 => 0,
                    0xa006..=0xa035 => self.cam.a006[addr as usize - 0xa006],
                    _ => panic!(),
                },
            },
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => {
                self.ram_enabled = data & 0xf == 0xa;
                self.ram_enabled = true;
            }
            0x2000..=0x3fff => self.rom_bank = (data as usize) & 0x3f,
            // Writing a value in range for 00h-0Fh maps the corresponding external RAM Bank to
            // memory at A000-BFFF. Writing any value with bit 5 set to '1' will select CAM
            // registers. Usually bank 10h is used to select the registers. All registers are
            // mirrored every 80h bytes. RAM bank 0 selected on reset.
            0x4000..=0x5fff => {
                if data & 0x10 == 0 {
                    self.mode = Mode::Ram;
                    self.ram_bank = (data & 0xf) as usize;
                } else {
                    self.mode = Mode::Cam;
                }
            }
            0xa000..=0xbfff => match self.mode {
                Mode::Ram if self.ram_enabled => {
                    self.ram[self.ram_bank][addr as usize - 0xa000] = data
                }
                Mode::Cam => match 0xa000 + (addr % 0x80) {
                    0xa000 => self.cam.a000 = data & 0x7,
                    0xa001 => self.cam.a001 = data,
                    0xa002 => self.cam.a002 = data,
                    0xa003 => self.cam.a003 = data,
                    0xa004 => self.cam.a004 = data,
                    0xa005 => self.cam.a005 = data,
                    0xa006..=0xa035 => self.cam.a006[addr as usize - 0xa006] = data,
                    _ => panic!(),
                },
                _ => {}
            },
            _ => panic!(),
        }

        // capture image
        if self.mode == Mode::Cam && self.cam.a000 & 0x1 == 1 {
            self.capture();
            self.cam.a000 ^= 1;
        }
    }
}

impl<S: Sensor> Controller for PoketCamera<S> {}
