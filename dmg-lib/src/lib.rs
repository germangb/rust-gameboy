//#![deny(dead_code)]
//#![deny(unused_imports)]
//#![deny(unused_must_use)]
//#![deny(unused_variables)]
//#![deny(unused_mut)]
//#![deny(unused_imports)]
//#![deny(clippy::style)]
//#![deny(clippy::correctness)]
//#![deny(clippy::complexity)]
//#![deny(clippy::perf)]

use apu::AudioOutput;
use cartridge::Cartridge;
use cpu::Cpu;
use dev::Device;
use mmu::Mmu;
use ppu::{palette::Palette, VideoOutput};

pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod dev;
pub mod interrupts;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod reg;
pub mod timer;
pub mod vram;
pub mod wram;

/// Main clock frequency.
const CLOCK: u64 = 4_194_304;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Mode {
    GB,
    CGB,
}

pub struct Dmg<C: Cartridge, V: VideoOutput, A: AudioOutput> {
    mode: Mode,
    cpu: Cpu,
    mmu: Box<Mmu<C, V, A>>,
    carry: u64,
}

impl<C: Cartridge, V: VideoOutput, A: AudioOutput> Dmg<C, V, A> {
    pub fn new(cartridge: C, mode: Mode, video_out: V, audio_out: A) -> Self {
        Self {
            mode,
            cpu: Cpu::default(),
            mmu: Box::new(Mmu::with_cartridge_video_audio(
                cartridge, mode, video_out, audio_out,
            )),
            carry: 0,
        }
    }

    pub fn emulate_frame(&mut self) {
        self.carry = self.mmu.emulate_frame(&mut self.cpu, self.carry);
    }

    /// Return the mode of emulation (GB or CGB).
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Return the Memory Manager Unit (MMU).
    pub fn mmu(&self) -> &Mmu<C, V, A> {
        &self.mmu
    }

    /// Return the Memory Manager Unit (MMU) as mutable.
    pub fn mmu_mut(&mut self) -> &mut Mmu<C, V, A> {
        &mut self.mmu
    }

    /// Return the CPU.
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Return the CPU as mutable.
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }
}

pub struct Builder<C: Cartridge, V: VideoOutput, A: AudioOutput> {
    mode: Option<Mode>,
    palette: Option<Palette>,
    skip_boot: bool,
    cartridge: C,
    video: V,
    audio: A,
}

impl Default for Builder<(), (), ()> {
    fn default() -> Self {
        Self {
            mode: None,
            palette: None,
            skip_boot: false,
            cartridge: (),
            video: (),
            audio: (),
        }
    }
}

impl<C: Cartridge, V: VideoOutput, A: AudioOutput> Builder<C, V, A> {
    pub fn with_cartridge<C2: Cartridge>(self, cartridge: C2) -> Builder<C2, V, A> {
        Builder {
            mode: self.mode,
            skip_boot: self.skip_boot,
            palette: self.palette,
            cartridge,
            video: self.video,
            audio: self.audio,
        }
    }

    pub fn with_video<V2: VideoOutput>(self, video: V2) -> Builder<C, V2, A> {
        Builder {
            mode: self.mode,
            skip_boot: self.skip_boot,
            palette: self.palette,
            cartridge: self.cartridge,
            video,
            audio: self.audio,
        }
    }

    pub fn with_audio<A2: AudioOutput>(self, audio: A2) -> Builder<C, V, A2> {
        Builder {
            mode: self.mode,
            skip_boot: self.skip_boot,
            palette: self.palette,
            cartridge: self.cartridge,
            video: self.video,
            audio,
        }
    }

    /// Set the default color palette on GB mode.
    ///
    /// The palette can be modified afterwards from the PPU at any time. This is
    /// just a way to initialize it to something other than GRAYSCALE values.
    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Disable boot rom. If the crate is not built using the *boot* feature
    /// flag, this is a no-op as the boot rom will be always skipped.
    pub fn skip_boot(mut self) -> Self {
        self.skip_boot = true;
        self
    }

    /// Set the preferred mode.
    ///
    /// By default, games that support both GB and CGB modes will default to
    /// using CGB if not otherwise specified by this method.
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn build(self) -> Dmg<C, V, A> {
        let cartridge = self.cartridge;
        let mode = self.mode.unwrap_or(Mode::CGB);
        let video = self.video;
        let audio = self.audio;
        let mut dmg = Dmg {
            mode,
            cpu: Cpu::default(),
            mmu: Box::new(Mmu::with_cartridge_video_audio(
                cartridge, mode, video, audio,
            )),
            carry: 0,
        };
        if self.skip_boot || cfg!(not(feature = "boot")) {
            let cpu = dmg.cpu_mut();

            cpu.reg_mut().set_af(0x01b0);
            cpu.reg_mut().set_bc(0x0013);
            cpu.reg_mut().set_de(0x00d8);
            cpu.reg_mut().set_hl(0x014d);
            cpu.reg_mut().sp = 0xfffe;
            cpu.reg_mut().pc = 0x0100;

            if let Mode::CGB = mode {
                cpu.reg_mut().a = 0x11;
            }

            let mmu = dmg.mmu_mut();

            mmu.write(0xFF05, 0x00); // TIMA
            mmu.write(0xFF06, 0x00); // TMA
            mmu.write(0xFF07, 0x00); // TAC
            mmu.write(0xFF10, 0x80); // NR10
            mmu.write(0xFF11, 0xBF); // NR11
            mmu.write(0xFF12, 0xF3); // NR12
            mmu.write(0xFF14, 0xBF); // NR14
            mmu.write(0xFF16, 0x3F); // NR21
            mmu.write(0xFF17, 0x00); // NR22
            mmu.write(0xFF19, 0xBF); // NR24
            mmu.write(0xFF1A, 0x7F); // NR30
            mmu.write(0xFF1B, 0xFF); // NR31
            mmu.write(0xFF1C, 0x9F); // NR32
            mmu.write(0xFF1E, 0xBF); // NR33
            mmu.write(0xFF20, 0xFF); // NR41
            mmu.write(0xFF21, 0x00); // NR42
            mmu.write(0xFF22, 0x00); // NR43
            mmu.write(0xFF23, 0xBF); // NR30
            mmu.write(0xFF24, 0x77); // NR50
            mmu.write(0xFF25, 0xF3); // NR51
            mmu.write(0xFF26, 0xF1); // NR52
            mmu.write(0xFF40, 0x91); // LCDC
            mmu.write(0xFF42, 0x00); // SCY
            mmu.write(0xFF43, 0x00); // SCX
            mmu.write(0xFF45, 0x00); // LYC
            mmu.write(0xFF47, 0xFC); // BGP
            mmu.write(0xFF48, 0xFF); // OBP0
            mmu.write(0xFF49, 0xFF); // OBP1
            mmu.write(0xFF4A, 0x00); // WY
            mmu.write(0xFF4B, 0x00); // WX
            mmu.write(0xFFFF, 0x00); // IE
            mmu.write(0xFF50, 0x01); // BOOT
        }
        if let Some(pal) = self.palette {
            dmg.mmu_mut().ppu_mut().set_palette(pal);
        }
        dmg
    }
}
