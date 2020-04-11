use crate::{
    apu::{device::Audio, Apu},
    cartridge::Cartridge,
    cpu::Cpu,
    int::Interrupts,
    joypad::Joypad,
    map::Mapped,
    ppu::{Ppu, Video},
    timer::Timer,
    wram::WRam,
    Mode, CLOCK,
};

// return value for the HDMA5 register some games expect all the bits to be set,
// even though the specification only requires the MSB to be.
//
// Tested games:
// - Simpsons THOH expectds 0xff to load levels (window now shown)
// - 0xff corrupts pokemon crystal
const HDMA5_DATA: u8 = 0xff;
const HDMA_DATA: u8 = 0xff; // HDMA1..4
const HRAM_SIZE: usize = 0x7f;

/// HRam memory
pub type HRam = Box<[u8; HRAM_SIZE]>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Speed {
    X1 = 0x00,
    X2 = 0x80,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct VRamDma {
    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
}

impl Default for VRamDma {
    fn default() -> Self {
        Self {
            hdma1: 0,
            hdma2: 0,
            hdma3: 0,
            hdma4: 0,
        }
    }
}

// 0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
// 4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
// 8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
// A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
// C000-CFFF   4KB Work RAM Bank 0 (WRAM)
// D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
// E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
// FE00-FE9F   Sprite Attribute Table (OAM)
// FEA0-FEFF   Not Usable
// FF00-FF7F   I/O Ports
// FF80-FFFE   High RAM (HRAM)
// FFFF        Interrupt Enable Register
pub struct Mmu<C: Cartridge, V: Video, D: Audio> {
    #[cfg_attr(not(feature = "boot"), allow(dead_code))]
    mode: Mode,
    boot: bool,
    cartridge: C,
    ppu: Ppu<V>,
    apu: Apu<D>,
    timer: Timer,
    wram: WRam,
    joy: Joypad,
    hram: HRam,
    vram_dma: VRamDma,
    int: Interrupts,
    speed: Speed,
}

impl<C: Cartridge, V: Video, D: Audio> Mmu<C, V, D> {
    pub fn new(mode: Mode, cartridge: C, video_out: V) -> Self {
        Self {
            mode,
            cartridge,
            boot: false,
            ppu: Ppu::new(mode, video_out),
            timer: Timer::default(),
            wram: WRam::default(),
            joy: Joypad::default(),
            apu: Apu::default(),
            hram: Box::new([0; HRAM_SIZE]),
            vram_dma: VRamDma::default(),
            int: Interrupts::default(),
            speed: Speed::X1,
        }
    }

    pub fn cartridge(&self) -> &C {
        &self.cartridge
    }

    pub fn cartridge_mut(&mut self) -> &mut C {
        &mut self.cartridge
    }

    pub fn joypad(&self) -> &Joypad {
        &self.joy
    }

    pub fn joypad_mut(&mut self) -> &mut Joypad {
        &mut self.joy
    }

    pub fn ppu(&self) -> &Ppu<V> {
        &self.ppu
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu<V> {
        &mut self.ppu
    }

    pub fn wram(&self) -> &WRam {
        &self.wram
    }

    pub fn wram_mut(&mut self) -> &mut WRam {
        &mut self.wram
    }

    pub fn apu(&self) -> &Apu<D> {
        &self.apu
    }

    pub fn apu_mut(&mut self) -> &mut Apu<D> {
        &mut self.apu
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn hram(&self) -> &HRam {
        &self.hram
    }

    pub fn hram_mut(&mut self) -> &mut HRam {
        &mut self.hram
    }

    pub(crate) fn emulate_frame(&mut self, cpu: &mut Cpu, carry: u64) -> u64 {
        const FRAME_CYCLES: u64 = CLOCK / 60;

        let mut cycles = carry;
        let mut cpu_rem = 0;
        while cycles < FRAME_CYCLES {
            let cpu_cycles = cpu.step(self);

            match self.speed {
                Speed::X1 => {
                    self.step(cpu_rem + cpu_cycles);
                    cycles += cpu_cycles;
                }
                Speed::X2 => {
                    let cpu_cycles = cpu_cycles + cpu_rem;
                    self.step(cpu_cycles / 2);
                    cpu_rem = cpu_cycles % 2;
                    cycles += cpu_cycles / 2;
                }
            }
        }

        // return carry. This value should be passed as carry argument on the next call
        // to this method.
        cycles % FRAME_CYCLES
    }

    // Advance the mapped components by the given amount of cycles of the internal
    // 4MHz clock.
    fn step(&mut self, cycles: u64) {
        if let Some(int) = self.joy.take_int() {
            self.int.set(int);
        }
        self.ppu.step(cycles);
        self.timer.step(cycles);
        self.apu.lock().step(cycles);

        // request generated interrupts
        if let Some(flag) = self.ppu.take_vblank_int() {
            self.int.set(flag);
        }
        if let Some(flag) = self.ppu.take_lcdc_int() {
            self.int.set(flag);
        }
        if let Some(flag) = self.timer.take_timer_int() {
            self.int.set(flag);
        }
    }

    // Writing to this register launches a DMA transfer from ROM or RAM to OAM
    // memory (sprite attribute table). The written value specifies the transfer
    // source address divided by 100h, ie. source & destination are:
    //
    // Source:      XX00-XX9F   ;XX in range from 00-F1h
    // Destination: FE00-FE9F
    //
    // It takes 160 microseconds until the transfer has completed (80 microseconds
    // in CGB Double Speed Mode), during this time the CPU can access only HRAM
    // (memory at FF80-FFFE). For this reason, the programmer must copy a short
    // procedure into HRAM, and use this procedure to start the transfer from inside
    // HRAM, and wait until the transfer has finished:
    fn oam_dma(&mut self, d: u8) {
        let src = u16::from(d) << 8;
        let dst = 0xfe00;

        for addr in 0..=0x9f {
            let src = src | (addr as u16);
            let dst = dst | (addr as u16);
            self.write(dst, self.read(src));
        }
    }

    // Writing to FF55 starts the transfer, the lower 7 bits of FF55 specify the
    // Transfer Length (divided by 10h, minus 1). Ie. lengths of 10h-800h bytes can
    // be defined by the values 00h-7Fh. And the upper bit of FF55 indicates the
    // Transfer Mode:
    fn vram_dma(&mut self, hdma5: u8) {
        let hdma1 = self.vram_dma.hdma1;
        let hdma2 = self.vram_dma.hdma2;
        let hdma3 = self.vram_dma.hdma3;
        let hdma4 = self.vram_dma.hdma4;

        let src = (u16::from(hdma1) << 8) | u16::from(hdma2 & 0xf0);
        let dst = 0x8000 | (u16::from(hdma3 & 0x1f) << 8) | u16::from(hdma4 & 0xf0);
        let len = (u16::from(hdma5 & 0x7f) + 1) * 16;

        let src = src..src + len;
        let dst = dst..dst + len;
        for (src, dst) in src.zip(dst) {
            let src = self.read(src);
            self.write(dst, src);
        }
    }
}

impl<C: Cartridge, V: Video, D: Audio> Mapped for Mmu<C, V, D> {
    fn read(&self, addr: u16) -> u8 {
        #[cfg(feature = "boot")]
        use dmg_boot::{cgb, gb};

        match addr {
            #[cfg(feature = "boot")]
            addr if !self.boot && self.mode == Mode::CGB && cgb::is_boot(addr) => {
                cgb::ROM[addr as usize]
            }
            #[cfg(feature = "boot")]
            addr if !self.boot && self.mode == Mode::GB && gb::is_boot(addr) => {
                gb::ROM[addr as usize]
            }

            0x0000..=0x7fff => self.cartridge.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr),
            0xa000..=0xbfff => self.cartridge.read(addr),
            0xc000..=0xdfff => self.wram.read(addr),
            0xe000..=0xfdff => self.wram.read(addr),
            0xfe00..=0xfe9f => self.ppu.read(addr),
            0xfea0..=0xfeff => 0,
            0xff00..=0xff7f => match addr {
                0xff00 => self.joy.read(addr),
                0xff01 | 0xff02 => 0,
                0xff04..=0xff07 => self.timer.read(addr),
                0xff0f => self.int.read(addr),
                0xff10..=0xff14
                | 0xff16..=0xff19
                | 0xff1a..=0xff1e
                | 0xff30..=0xff3f
                | 0xff20..=0xff26
                | 0xff27..=0xff2f => self.apu.read(addr),
                0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f | 0xff68..=0xff6b => self.ppu.read(addr),
                0xff46 => 0, // OAM DMA
                0xff50 => 0,
                0xff51..=0xff54 => HDMA_DATA,
                0xff55 => HDMA5_DATA,
                0xff4d => self.speed as u8,
                0xff70 => self.wram.read(addr),
                _ => {
                    //println!("ERROR {:04x}", addr);
                    0
                }
            },
            0xff80..=0xfffe => self.hram[addr as usize - 0xff80],
            0xffff => self.int.read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        #[cfg(feature = "boot")]
        use dmg_boot::{cgb, gb};

        match addr {
            #[cfg(feature = "boot")]
            addr if !self.boot && self.mode == Mode::CGB && cgb::is_boot(addr) => {}
            #[cfg(feature = "boot")]
            addr if !self.boot && self.mode == Mode::GB && gb::is_boot(addr) => {}

            0x0000..=0x7fff => self.cartridge.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data),
            0xa000..=0xbfff => self.cartridge.write(addr, data),
            0xc000..=0xdfff => self.wram.write(addr, data),
            0xe000..=0xfdff => self.wram.write(addr, data),
            0xfe00..=0xfe9f => self.ppu.write(addr, data),
            0xfea0..=0xfeff => { /* Not Usable */ }
            0xff00..=0xff7f => match addr {
                0xff00 => self.joy.write(addr, data),
                0xff01 | 0xff02 => { /* serial data transfer (link cable) */ }
                0xff04..=0xff07 => self.timer.write(addr, data),
                0xff0f => self.int.write(addr, data),
                0xff10..=0xff14
                | 0xff16..=0xff19
                | 0xff1a..=0xff1e
                | 0xff30..=0xff3f
                | 0xff20..=0xff26
                | 0xff27..=0xff2f => self.apu.write(addr, data),
                0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f | 0xff68..=0xff6b => {
                    self.ppu.write(addr, data)
                }
                0xff46 => self.oam_dma(data),
                0xff50 => {
                    if !self.boot {
                        self.boot = data & 0x1 != 0;
                    }
                }
                0xff51 => self.vram_dma.hdma1 = data,
                0xff52 => self.vram_dma.hdma2 = data,
                0xff53 => self.vram_dma.hdma3 = data,
                0xff54 => self.vram_dma.hdma4 = data,
                0xff55 => self.vram_dma(data),

                // KEY1
                0xff4d => {
                    if data & 0x1 != 0 {
                        self.speed = Speed::X2;
                    }
                }
                0xff70 => self.wram.write(addr, data),
                _ => {}
            },
            0xff80..=0xfffe => self.hram[addr as usize - 0xff80] = data,
            0xffff => self.int.write(addr, data),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{map::Mapped, mmu::Mmu, Mode};

    #[test]
    fn oam_dma() {
        let mut mmu = Mmu::<_, _, ()>::new(Mode::GB, (), ());

        mmu.write(0xff46, 0);

        for addr in 0..=0x9f {
            let rom = mmu.read(addr as u16);
            let oam = mmu.read(0xfe00 | (addr as u16));
            assert_eq!(rom, oam);
        }
    }
}
