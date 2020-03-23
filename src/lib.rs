// #![deny(dead_code)]
// #![deny(unused_imports)]
// #![deny(unused_must_use)]
// #![deny(unused_variables)]
// #![deny(unused_mut)]
// #![deny(unused_imports)]
// #![warn(clippy::style)]
// #![deny(clippy::correctness)]
// #![deny(clippy::complexity)]
// #![deny(clippy::perf)]

use crate::{
    cartridge::Cartridge,
    cpu::Cpu,
    dev::Device,
    mmu::Mmu,
    ppu::{VideoOutput, HBLANK_CYCLES, OAM_CYCLES, PIXEL_CYCLES},
};

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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Mode {
    GB,
    CGB,
}

pub struct Dmg<V> {
    mode: Mode,
    cpu: Cpu,
    mmu: Box<Mmu<V>>,
    carry: usize,
}

impl<V> Dmg<V> {
    pub fn new<C: Cartridge + 'static>(cartridge: C, mode: Mode, output: V) -> Self {
        Self {
            mode,
            cpu: Cpu::default(),
            mmu: Box::new(Mmu::new(cartridge, mode, output)),
            carry: 0,
        }
    }

    pub fn mmu(&self) -> &Mmu<V> {
        &self.mmu
    }

    pub fn mmu_mut(&mut self) -> &mut Mmu<V> {
        &mut self.mmu
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }
}

impl<V: VideoOutput> Dmg<V> {
    pub fn emulate_frame(&mut self) {
        let frame_ticks = (OAM_CYCLES + PIXEL_CYCLES + HBLANK_CYCLES) * 154;
        let mut cycles = 0;
        while cycles < frame_ticks {
            let cpu_cycles = self.cpu.step(&mut self.mmu);
            self.mmu.step(cpu_cycles);
            cycles += cpu_cycles;
        }
        self.carry = cycles % frame_ticks;
    }

    pub fn boot(&mut self) {
        self.boot_gb();
        if let Mode::CGB = self.mode {
            self.cpu.reg_mut().a = 0x11;
        }
    }

    fn boot_gb(&mut self) {
        // When the GameBoy is powered up, a 256 byte program starting at memory
        // location 0 is executed. This program is located in a ROM inside the GameBoy.
        // The first thing the program does is read the cartridge locations from $104 to
        // $133 and place this graphic of a Nintendo logo on the screen at the top. This
        // image is then scrolled until it is in the middle of the screen. Two musical
        // notes are then played on the internal speaker. Again, the cartridge locations
        // $104 to $133 are read but this time they are compared with a table in the
        // internal rom. If any byte fails to compare, then the GameBoy stops comparing
        // bytes and simply halts all operations. If all locations compare the same,
        // then the GameBoy starts adding all of the bytes in the cartridge from $134 to
        // $14d. A value of 25 decimal is added to this total. If the least significant
        // byte of the result is a not a zero, then the GameBoy will stop doing
        // anything. If it is a zero, then the internal ROM is disabled and cartridge
        // program execution begins at location $100 with the following register values:
        self.cpu.reg_mut().set_af(0x01b0);
        self.cpu.reg_mut().set_bc(0x0013);
        self.cpu.reg_mut().set_de(0x00d8);
        self.cpu.reg_mut().set_hl(0x014d);
        self.cpu.reg_mut().sp = 0xfffe;
        self.cpu.reg_mut().pc = 0x0100;

        self.mmu.write(0xFF05, 0x00); // TIMA
        self.mmu.write(0xFF06, 0x00); // TMA
        self.mmu.write(0xFF07, 0x00); // TAC
        self.mmu.write(0xFF10, 0x80); // NR10
        self.mmu.write(0xFF11, 0xBF); // NR11
        self.mmu.write(0xFF12, 0xF3); // NR12
        self.mmu.write(0xFF14, 0xBF); // NR14
        self.mmu.write(0xFF16, 0x3F); // NR21
        self.mmu.write(0xFF17, 0x00); // NR22
        self.mmu.write(0xFF19, 0xBF); // NR24
        self.mmu.write(0xFF1A, 0x7F); // NR30
        self.mmu.write(0xFF1B, 0xFF); // NR31
        self.mmu.write(0xFF1C, 0x9F); // NR32
        self.mmu.write(0xFF1E, 0xBF); // NR33
        self.mmu.write(0xFF20, 0xFF); // NR41
        self.mmu.write(0xFF21, 0x00); // NR42
        self.mmu.write(0xFF22, 0x00); // NR43
        self.mmu.write(0xFF23, 0xBF); // NR30
        self.mmu.write(0xFF24, 0x77); // NR50
        self.mmu.write(0xFF25, 0xF3); // NR51
        self.mmu.write(0xFF26, 0xF1); // NR52
        self.mmu.write(0xFF40, 0x91); // LCDC
        self.mmu.write(0xFF42, 0x00); // SCY
        self.mmu.write(0xFF43, 0x00); // SCX
        self.mmu.write(0xFF45, 0x00); // LYC
        self.mmu.write(0xFF47, 0xFC); // BGP
        self.mmu.write(0xFF48, 0xFF); // OBP0
        self.mmu.write(0xFF49, 0xFF); // OBP1
        self.mmu.write(0xFF4A, 0x00); // WY
        self.mmu.write(0xFF4B, 0x00); // WX
        self.mmu.write(0xFFFF, 0x00); // IE
        self.mmu.write(0xFF50, 0x01); // BOOT
    }
}
