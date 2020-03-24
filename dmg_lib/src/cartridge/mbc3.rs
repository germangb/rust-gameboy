use crate::dev::Device;

enum Mode {
    Ram,
    Rtc,
}

#[rustfmt::skip]
pub struct Mbc3 {
    rom: Box<[u8]>,
    ram: Vec<[u8; 0x2000]>,
    // The Clock Counter Registers
    // 08h  RTC S   Seconds   0-59 (0-3Bh)
    // 09h  RTC M   Minutes   0-59 (0-3Bh)
    // 0Ah  RTC H   Hours     0-23 (0-17h)
    // 0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
    // 0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
    //         Bit 0  Most significant bit of Day Counter (Bit 8)
    //         Bit 6  Halt (0=Active, 1=Stop Timer)
    //         Bit 7  Day Counter Carry Bit (1=Counter Overflow)
    rtc: [u8; 5],
    rtc_select: usize,
    rom_bank: usize,
    ram_bank: usize,
    ram_timer_enabled: bool,
    mode: Mode,
}

impl Mbc3 {
    pub fn from_bytes<B: Into<Box<[u8]>>>(rom: B) -> Self {
        let ram_banks = 4;
        Self {
            rom: rom.into(),
            ram: vec![[0; 0x2000]; ram_banks],
            rtc: [0; 5],
            rtc_select: 0,
            rom_bank: 0,
            ram_bank: 0,
            ram_timer_enabled: false,
            mode: Mode::Ram,
        }
    }
}

impl Device for Mbc3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.rom[0x4000 * self.rom_bank.max(1) + addr as usize - 0x4000],
            0xa000..=0xbfff => {
                if self.ram_timer_enabled {
                    match self.mode {
                        Mode::Ram => self.ram[self.ram_bank][addr as usize - 0xa000],
                        Mode::Rtc => self.rtc[self.rtc_select],
                    }
                } else {
                    0
                }
            }
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => self.ram_timer_enabled = data & 0xf == 0xa,
            0x2000..=0x3fff => self.rom_bank = data as usize,
            // As for the MBC1s RAM Banking Mode, writing a value in range for 00h-03h maps the
            // corresponding external RAM Bank (if any) into memory at A000-BFFF.
            //
            // When writing a value of 08h-0Ch, this will map the corresponding RTC register into
            // memory at A000-BFFF. That register could then be read/written by accessing any
            // address in that area, typically that is done by using address A000.
            0x4000..=0x5fff => match data {
                0x00..=0x03 => {
                    self.mode = Mode::Ram;
                    self.ram_bank = data as usize
                }
                0x08..=0x0c => {
                    self.mode = Mode::Rtc;
                    self.rtc_select = data as usize - 0x08
                }
                _ => panic!(),
            },
            // When writing 00h, and then 01h to this register, the current time becomes latched
            // into the RTC registers. The latched data will not change until it becomes latched
            // again, by repeating the write 00h->01h procedure.
            //
            // This is supposed for <reading> from the RTC registers. It is proof to read the
            // latched (frozen) time from the RTC registers, while the clock itself continues to
            // tick in background.
            0x6000..=0x7fff => {}
            // Depending on the current Bank Number/RTC Register selection (see below), this memory
            // space is used to access an 8KByte external RAM Bank, or a single RTC Register.
            0xa000..=0xbfff => {
                if self.ram_timer_enabled {
                    match self.mode {
                        Mode::Ram => self.ram[self.ram_bank][addr as usize - 0xa000] = data,
                        Mode::Rtc => self.rtc[self.rtc_select] = data,
                    }
                }
            }
            _ => panic!(),
        }
    }
}
