use crate::{
    apu::{Apu, AudioOutput},
    cartridge::Cartridge,
    cpu::Cpu,
    dev::Device,
    interrupts::Interrupts,
    joypad::Joypad,
    ppu::{Ppu, VideoOutput, HBLANK_CYCLES, OAM_CYCLES, PIXEL_TRANSFER_CYCLES, VBLANK_CYCLES},
    timer::Timer,
    wram::WorkRam,
    Mode,
};

// return value for the HDMA5 register some games expect all the bits to be set,
// even though the specification only requires the MSB to be.
//
// Tested games:
// - Simpsons THOH expectds 0xff to load levels (window now shown)
// - 0xff corrupts pokemon crystal
const HDMA5_DATA: u8 = 0xff;
const HDMA_DATA: u8 = 0xff; // HDMA1..4
const UNUSED_DATA: u8 = 0x00;
const UNHANDLED_DATA: u8 = 0x00;
const BOOT_REG_DATA: u8 = 0x00; // ff50
const HRAM_SIZE: usize = 0x7f;

/// HRam memory
pub type HRam = [u8; HRAM_SIZE];

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Speed {
    X1 = 0x00,
    X2 = 0x80,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct VRamDma {
    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
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
pub struct Mmu<C: Cartridge, V: VideoOutput, A: AudioOutput> {
    #[cfg_attr(not(feature = "boot"), allow(dead_code))]
    mode: Mode,
    boot: bool,
    cartridge: C,
    ppu: Ppu<V>,
    apu: Apu<A>,
    timer: Timer,
    wram: WorkRam,
    joy: Joypad,
    hram: HRam,
    vram_dma: VRamDma,
    int: Interrupts,
    // FF4D - KEY1 - CGB Mode Only - Prepare Speed Switch
    //
    // Bit 7: Current Speed     (0=Normal, 1=Double) (Read Only)
    // Bit 0: Prepare Speed Switch (0=No, 1=Prepare) (Read/Write)
    speed: Speed,
}

impl<C: Cartridge, V: VideoOutput, A: AudioOutput> Mmu<C, V, A> {
    pub fn with_cartridge_video_audio(
        cartridge: C,
        mode: Mode,
        video_out: V,
        audio_out: A,
    ) -> Self {
        let vram_dma = VRamDma {
            hdma1: 0,
            hdma2: 0,
            hdma3: 0,
            hdma4: 0,
        };
        let ppu = Ppu::with_mode_and_video(mode, video_out);
        let timer = Timer::default();
        let wram = WorkRam::default();
        let int = Interrupts::default();
        let joy = Joypad::default();
        let apu = Apu::with_audio(audio_out);
        let speed = Speed::X1;
        let hram = [0; HRAM_SIZE];
        Self {
            mode,
            boot: false,
            cartridge,
            ppu,
            timer,
            wram,
            joy,
            apu,
            hram,
            vram_dma,
            int,
            speed,
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

    pub fn wram(&self) -> &WorkRam {
        &self.wram
    }

    pub fn wram_mut(&mut self) -> &mut WorkRam {
        &mut self.wram
    }

    pub fn apu(&self) -> &Apu<A> {
        &self.apu
    }

    pub fn apu_mut(&mut self) -> &mut Apu<A> {
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
        let frame_ticks =
            (OAM_CYCLES + PIXEL_TRANSFER_CYCLES + HBLANK_CYCLES) * 144 + VBLANK_CYCLES;

        let mut cycles = carry;
        let mut cpu_rem = 0;
        while cycles < frame_ticks {
            let cpu_cycles = cpu.step(self);

            match self.speed {
                Speed::X2 => {
                    let cpu_cycles = cpu_cycles + cpu_rem;
                    self.step(cpu_cycles / 2);
                    cpu_rem = cpu_cycles % 2;
                    cycles += cpu_cycles / 2;
                }
                Speed::X1 => {
                    self.step(cpu_rem + cpu_cycles);
                    cycles += cpu_cycles;
                }
            }
        }
        // return carry. This value should be passed as carry argument on the next call
        // to this method.
        cycles % frame_ticks
    }

    // Advance the mapped components by the given amount of cycles of the internal
    // 4MHz clock.
    fn step(&mut self, cycles: u64) {
        if let Some(int) = self.joy.take_int() {
            self.int.set(int);
        }
        self.ppu.step(cycles, &mut self.int);
        self.timer.step(cycles, &mut self.int);
        self.cartridge.step(cycles);
        self.apu.step(cycles);
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
        let len = 0x9f;

        #[cfg(feature = "logging")]
        log::info!(target: "mmu", "OAM DMA transfer. SRC = {:#04x}, DST = {:#04x}, LEN = {:#02x}", src, dst, len);

        for addr in 0..=len {
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

        #[cfg(feature = "logging")]
        log::info!(target: "mmu", "VRAM DMA transfer. SRC = {:#04x}, DST = {:#04x}, LEN = {:#04x}", src, dst, len);

        let src = src..src + len;
        let dst = dst..dst + len;
        for (src, dst) in src.zip(dst) {
            let src = self.read(src);
            self.write(dst, src);
        }
    }
}

impl<C: Cartridge, V: VideoOutput, A: AudioOutput> Device for Mmu<C, V, A> {
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
            0xfea0..=0xfeff => UNUSED_DATA,
            0xff00..=0xff7f => match addr {
                0xff00 => self.joy.read(addr),
                0xff01 | 0xff02 => UNUSED_DATA,
                0xff04..=0xff07 => self.timer.read(addr),
                0xff0f => self.int.read(addr),
                0xff10..=0xff14
                | 0xff16..=0xff19
                | 0xff1a..=0xff1e
                | 0xff30..=0xff3f
                | 0xff20..=0xff26 => self.apu.read(addr),
                0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f | 0xff68..=0xff6b => self.ppu.read(addr),
                0xff46 => UNUSED_DATA, // OAM DMA
                0xff50 => BOOT_REG_DATA,
                0xff51..=0xff54 => HDMA_DATA,
                0xff55 => HDMA5_DATA,
                0xff4d => self.speed as u8,
                0xff70 => self.wram.read(addr),
                _ => UNHANDLED_DATA,
            },
            0xff80..=0xfffe => match addr {
                0xff80..=0xfffe => self.hram[addr as usize - 0xff80],
                _ => panic!(),
            },
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
                | 0xff20..=0xff26 => self.apu.write(addr, data),
                0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f | 0xff68..=0xff6b => {
                    self.ppu.write(addr, data)
                }
                0xff46 => self.oam_dma(data),
                0xff50 => {
                    #[cfg(feature = "logging")]
                    log::info!(target: "mmu", "BOOT register = {:#02x}", data);

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
                        #[cfg(feature = "logging")]
                        log::info!(target: "mmu", "Double speed mode enabled");

                        self.speed = Speed::X2;
                    }
                }
                0xff70 => self.wram.write(addr, data),
                _ => {}
            },
            0xff80..=0xfffe => match addr {
                0xff80..=0xfffe => self.hram[addr as usize - 0xff80] = data,
                _ => panic!(),
            },
            0xffff => self.int.write(addr, data),
        }
    }
}

#[cfg(tests)]
mod tests {
    use crate::{
        cartridge::{Rom, ZeroRom},
        dev::Device,
        mmu::Mmu,
        Mode,
    };

    #[test]
    fn dma() {
        let mut mmu = Mmu::with_cartridge_video_audio((), Mode::GB, (), ());

        mmu.write(0xff50, 1);
        mmu.write(0xff46, 0);

        for addr in 0..=0x9f {
            let rom = mmu.read(addr as u16);
            let oam = mmu.read(0xfe00 | (addr as u16));
            assert_eq!(rom, oam);
        }
    }
}
