use crate::{dev::Device, CLOCK};
use log::info;

// TODO
const BUFFER_SIZE: usize = 0;

pub type Sample = i16;

pub trait AudioOutput {
    fn queue(&mut self, channel: usize, samples: &[Sample]);
    fn on(&mut self, channel: usize);
    fn off(&mut self, channel: usize);
    fn clear(&mut self, channel: usize);
}

impl AudioOutput for () {
    fn queue(&mut self, _: usize, _: &[Sample]) {}
    fn on(&mut self, _: usize) {}
    fn off(&mut self, _: usize) {}
    fn clear(&mut self, _: usize) {}
}

// convert clock cycles to nanos
#[allow(dead_code)]
fn cycles_to_nano(cycles: u64) -> u64 {
    cycles * 1_000_000_000 / CLOCK
}

// Convert nanos to clock cycles
#[allow(dead_code)]
fn nano_to_cycles(nano: u64) -> u64 {
    nano * CLOCK / 1_000_000_000
}

#[allow(dead_code)]
pub struct Apu<A: AudioOutput> {
    output: A,
    buffer: Box<[Sample; BUFFER_SIZE]>,
    // Sound Channel 1 - Tone & Sweep
    // Bit 6-4 - Sweep Time
    // Bit 3   - Sweep Increase/Decrease
    //            0: Addition    (frequency increases)
    //            1: Subtraction (frequency decreases)
    // Bit 2-0 - Number of sweep shift (n: 0-7)
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    // Sound Channel 2 - Tone
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    // Sound Channel 3 - Wave Output
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    wave_ram: [u8; 0x10],
    // Sound Channel 4 - Noise
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    // Sound Control Registers
    nr50: u8,
    // Bit 7 - Output sound 4 to SO2 terminal
    // Bit 6 - Output sound 3 to SO2 terminal
    // Bit 5 - Output sound 2 to SO2 terminal
    // Bit 4 - Output sound 1 to SO2 terminal
    // Bit 3 - Output sound 4 to SO1 terminal
    // Bit 2 - Output sound 3 to SO1 terminal
    // Bit 1 - Output sound 2 to SO1 terminal
    // Bit 0 - Output sound 1 to SO1 terminal
    nr51: u8,
    // Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
    // Bit 3 - Sound 4 ON flag (Read Only)
    // Bit 2 - Sound 3 ON flag (Read Only)
    // Bit 1 - Sound 2 ON flag (Read Only)
    // Bit 0 - Sound 1 ON flag (Read Only)
    nr52: u8,
}

impl<A: AudioOutput> Apu<A> {
    pub fn with_audio(output: A) -> Self {
        Self {
            output,
            buffer: Box::new([0; BUFFER_SIZE]),
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            wave_ram: [0; 0x10],
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr51: 0,
            nr52: 0,
        }
    }

    pub(crate) fn step(&mut self, _cycles: u64) {}
}

impl<A: AudioOutput> Device for Apu<A> {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff10 => self.nr10,
            0xff11 => self.nr11,
            0xff12 => self.nr12,
            0xff13 => self.nr13,
            0xff14 => self.nr14,

            0xff16 => self.nr21,
            0xff17 => self.nr22,
            0xff18 => self.nr23,
            0xff19 => self.nr24,

            0xff1a => self.nr30,
            0xff1b => self.nr31,
            0xff1c => self.nr32,
            0xff1d => self.nr33,
            0xff1e => self.nr34,
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30],

            0xff20 => self.nr41,
            0xff21 => self.nr42,
            0xff22 => self.nr43,
            0xff23 => self.nr44,

            0xff24 => self.nr50,
            0xff25 => self.nr51,
            0xff26 => self.nr52,
            _ => panic!(),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // Channel 1 sweep
            0xff10 => self.nr10 = data,
            0xff11 => self.nr11 = data,
            0xff12 => self.nr12 = data,
            0xff13 => self.nr13 = data,
            0xff14 => self.nr14 = data,

            // Channel 2 - Tone
            0xff16 => self.nr21 = data,
            0xff17 => self.nr22 = data,
            0xff18 => self.nr23 = data,
            0xff19 => self.nr24 = data,

            // Channel 3 - Wave RAM
            0xff1a => self.nr30 = data,
            0xff1b => self.nr31 = data,
            0xff1c => self.nr32 = data,
            0xff1d => self.nr33 = data,
            0xff1e => self.nr34 = data,
            0xff30..=0xff3f => self.wave_ram[addr as usize - 0xff30] = data,

            // Channel 4 - Noise
            0xff20 => self.nr41 = data,
            0xff21 => self.nr42 = data,
            0xff22 => self.nr43 = data,
            0xff23 => self.nr44 = data,

            0xff24 => self.nr50 = data,
            0xff25 => self.nr51 = data,
            0xff26 => {
                self.nr52 = data;
                info!("NR52 = {:08b}", data);

                if data & 0x80 != 0 {
                    self.output.on(0);
                }
            }
            _ => panic!(),
        }
    }
}
