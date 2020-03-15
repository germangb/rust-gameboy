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
use crate::{
    cartridge::Cartridge,
    cpu::Cpu,
    mmu::Mmu,
    ppu::{HBLANK, OAM, PIXEL},
};

pub mod cartridge;
pub mod cpu;
pub mod device;
pub mod interrupts;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod registers;
pub mod sound;
pub mod timer;

pub struct Dmg {
    cpu: Cpu,
    mmu: Box<Mmu>,
    carry: usize,
}

impl Dmg {
    pub fn new<C: Cartridge + 'static>(cartridge: C) -> Self {
        Self {
            cpu: Cpu::default(),
            mmu: Box::new(Mmu::new(cartridge)),
            carry: 0,
        }
    }

    pub fn emulate_frame(&mut self) {
        let frame_ticks = (OAM + PIXEL + HBLANK) * 153;
        let mut cycles = 0;
        while cycles < frame_ticks {
            let cpu_cycles = self.cpu.step(&mut self.mmu);
            self.mmu.step(cpu_cycles);
            cycles += cpu_cycles;
        }
        self.carry = cycles % frame_ticks;
    }

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    pub fn mmu_mut(&mut self) -> &mut Mmu {
        &mut self.mmu
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}
