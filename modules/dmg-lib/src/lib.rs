#![cfg_attr(not(debug_assertions),
            warn(dead_code, unused_imports, unused_variables))]
#![deny(clippy::style, clippy::correctness, clippy::complexity, clippy::perf)]
use crate::{
    apu::device::Audio, cartridge::Cartridge, cpu::Cpu, device::Device, mmu::Mmu, ppu::Video,
};
use std::marker::PhantomData;

pub mod apu;
pub mod cartridge;
mod clock;
pub mod cpu;
pub mod device;
pub mod interrupt;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod timer;
pub mod vram;
pub mod wram;

const CLOCK: u64 = 4_194_304;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Mode {
    GB,
    CGB,
}

// TODO consider not abusing generics.
pub struct GameBoy<C: Cartridge, V: Video, D: Audio> {
    cpu: Cpu,
    mmu: Mmu<C, V, D>,
    carry: u64,
}

impl Default for GameBoy<(), (), ()> {
    fn default() -> Self {
        Builder::default().build()
    }
}

impl<C: Cartridge, V: Video, D: Audio> GameBoy<C, V, D> {
    pub fn emulate_frame(&mut self) {
        self.carry = self.mmu.emulate_frame(&mut self.cpu, self.carry);
    }

    /// Return the Memory Manager Unit (MMU).
    pub fn mmu(&self) -> &Mmu<C, V, D> {
        &self.mmu
    }

    /// Return the Memory Manager Unit (MMU) as mutable.
    pub fn mmu_mut(&mut self) -> &mut Mmu<C, V, D> {
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

pub struct Builder<C: Cartridge, V: Video, D: Audio> {
    _phantom: PhantomData<D>,
    mode: Option<Mode>,
    skip_boot: bool,
    cartridge: C,
    video: V,
}

impl Default for Builder<(), (), ()> {
    fn default() -> Self {
        Self { _phantom: PhantomData,
               mode: None,
               skip_boot: false,
               cartridge: (),
               video: () }
    }
}

impl<C: Cartridge, V: Video, D: Audio> Builder<C, V, D> {
    pub fn audio<D2: Audio>(self) -> Builder<C, V, D2> {
        Builder { _phantom: PhantomData,
                  mode: self.mode,
                  skip_boot: self.skip_boot,
                  cartridge: self.cartridge,
                  video: self.video }
    }

    pub fn cartridge<C2: Cartridge>(self, cartridge: C2) -> Builder<C2, V, D> {
        Builder { _phantom: PhantomData,
                  mode: self.mode,
                  skip_boot: self.skip_boot,
                  cartridge,
                  video: self.video }
    }

    pub fn video<V2: Video>(self, video: V2) -> Builder<C, V2, D> {
        Builder { _phantom: PhantomData,
                  mode: self.mode,
                  skip_boot: self.skip_boot,
                  cartridge: self.cartridge,
                  video }
    }

    /// Disable dmg-data rom. If the crate is not built using the *dmg-data*
    /// feature flag, this is a no-op as the dmg-data rom will be always
    /// skipped.
    pub fn skip_boot(mut self) -> Self {
        self.skip_boot = true;
        self
    }

    fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set up for Color Game Boy emulation.
    pub fn gbc_mode(mut self) -> Self {
        self.with_mode(Mode::CGB)
    }

    /// Set up for Non-color Game Boy emulation.
    pub fn gb_mode(mut self) -> Self {
        self.with_mode(Mode::GB)
    }

    pub fn build(self) -> GameBoy<C, V, D> {
        let cartridge = self.cartridge;
        let mode = self.mode.unwrap_or(Mode::CGB);
        let video = self.video;
        let mut dmg = GameBoy { cpu: Cpu::default(),
                                mmu: Mmu::new(mode, cartridge, video),
                                carry: 0 };

        // FIXME Bugs:
        //  - GB game on CGB mode (color palette is not set).
        if self.skip_boot {
            let cpu = dmg.cpu_mut();

            // Initialize CPU
            cpu.reg_mut().set_af(0x01b0);
            cpu.reg_mut().set_bc(0x0013);
            cpu.reg_mut().set_de(0x00d8);
            cpu.reg_mut().set_hl(0x014d);
            cpu.reg_mut().sp = 0xfffe;
            cpu.reg_mut().pc = 0x0100;

            // Games that work on both GB and CGB check the A register to detect CGB
            // hardware and enhance games (mostly add color and better tiles).
            if let Mode::CGB = mode {
                cpu.reg_mut().a = 0x11;
            }

            let mmu = dmg.mmu_mut();

            // Initialize memory map
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
        dmg
    }
}
