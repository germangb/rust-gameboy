use crate::{
    cartridge::Cartridge,
    dev::Device,
    mmu::Mmu,
    ppu::VideoOutput,
    reg::{Flag::*, Registers},
};

static CYCLES: [u64; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, 0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1,
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, 2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2, 2, 0, 2, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, 2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4,
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, 3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4,
];

static CB_CYCLES: [u64; 256] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2,
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
];

#[derive(Debug)]
pub struct Cpu {
    reg: Registers,
    ime: bool,
    halt: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            reg: Registers::default(),
            ime: false,
            halt: false,
        }
    }
}

impl Cpu {
    pub fn reg(&self) -> &Registers {
        &self.reg
    }

    pub fn reg_mut(&mut self) -> &mut Registers {
        &mut self.reg
    }

    pub fn ime(&self) -> bool {
        self.ime
    }

    pub fn halt(&self) -> bool {
        self.halt
    }

    fn fetch<C: Cartridge, V: VideoOutput>(&mut self, mmu: &Mmu<C, V>) -> u8 {
        let b = mmu.read(self.reg.pc);
        self.reg.pc += 1;
        b
    }

    fn fetch_word<C: Cartridge, V: VideoOutput>(&mut self, mmu: &Mmu<C, V>) -> u16 {
        let lo = mmu.read(self.reg.pc) as u16;
        let hi = mmu.read(self.reg.pc + 1) as u16;
        self.reg.pc += 2;
        (hi << 8) | lo
    }

    fn fetch_signed<C: Cartridge, V: VideoOutput>(&mut self, mmu: &Mmu<C, V>) -> i8 {
        let n: i8 = unsafe { std::mem::transmute(self.fetch(mmu)) };
        n
    }

    // Pushes word into the stack
    // Decrements SP by 2
    fn stack_push<C: Cartridge, V: VideoOutput>(&mut self, nn: u16, mmu: &mut Mmu<C, V>) {
        self.reg.sp -= 2;
        mmu.write_word(self.reg.sp, nn);
    }

    // Pops word from the stack
    // Increments SP by 2
    fn stack_pop<C: Cartridge, V: VideoOutput>(&mut self, mmu: &Mmu<C, V>) -> u16 {
        let r = mmu.read_word(self.reg.sp);
        self.reg.sp += 2;
        r
    }

    // Add n to A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 H C
    fn add_n(&mut self, n: u8) {
        let res = u16::from(self.reg.a) + u16::from(n);
        self.reg.set_flag(Z, res.trailing_zeros() >= 8);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (self.reg.a & 0xf) + (n & 0xf) > 0xf);
        self.reg.set_flag(C, res > 0xff);
        self.reg.a = (res & 0xff) as u8;
    }

    // Add n + Carry flag to A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 H C
    fn adc_n(&mut self, n: u8) {
        let mut res = u16::from(self.reg.a) + u16::from(n);
        let carry = if self.reg.is_flag(C) { 1 } else { 0 };
        res += u16::from(carry);

        self.reg.set_flag(Z, res.trailing_zeros() >= 8);
        self.reg.set_flag(N, false);
        self.reg
            .set_flag(H, (self.reg.a & 0xf) + (n & 0xf) + carry > 0xf);
        self.reg.set_flag(C, res > 0xff);
        self.reg.a = (res & 0xff) as u8;
    }

    // Subtract n from A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn sub_n(&mut self, n: u8) {
        let res = self.reg.a.wrapping_sub(n);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, n & 0xf > self.reg.a & 0xf);
        self.reg.set_flag(C, n > self.reg.a);
        self.reg.a = res;
    }
    // Subtract n + Carry flag from A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn sbc_n(&mut self, n: u8) {
        let c = if self.reg.is_flag(C) { 1 } else { 0 };
        let res = self.reg.a.wrapping_sub(n).wrapping_sub(c);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (n & 0xf) + c > self.reg.a & 0xf);
        self.reg
            .set_flag(C, u16::from(n) + u16::from(c) > u16::from(self.reg.a));
        self.reg.a = res;
    }

    // Logically AND n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 1 0
    fn and_n(&mut self, n: u8) {
        let res = self.reg.a & n;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    // Logically OR n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 1 0
    fn or_n(&mut self, n: u8) {
        let res = self.reg.a | n;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    // Logically XOR n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 0 0
    fn xor_n(&mut self, n: u8) {
        let res = self.reg.a ^ n;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    // Compare A with n.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn cp_n(&mut self, n: u8) {
        let a = self.reg.a;
        self.sub_n(n);
        self.reg.a = a;
    }

    // Increment register n.
    // n = A,B,C,D,E,H,(HL)
    // Flags
    // Z 0 H -
    fn inc_n(&mut self, n: u8) -> u8 {
        let res = n.wrapping_add(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, n & 0xf == 0xf);
        res
    }

    // Decrement register n.
    // n = A,B,C,D,E,H,(HL)
    fn dec_n(&mut self, n: u8) -> u8 {
        let res = n.wrapping_sub(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, n.trailing_zeros() >= 4);
        res
    }

    // Add n to HL.
    // n = BC,DE,HL,SP
    // Flags
    // - 0 H C
    fn add_hl_nn(&mut self, nn: u16) {
        let res = u32::from(self.reg.hl()) + u32::from(nn);
        self.reg.set_flag(N, false);
        self.reg
            .set_flag(H, (self.reg.hl() & 0xfff) + (nn & 0xfff) > 0xfff);
        self.reg
            .set_flag(C, u32::from(self.reg.hl()) + u32::from(nn) > 0xffff);
        self.reg.set_hl((res & 0xffff) as u16);
    }

    // Add n to Stack Pointer (SP).
    // n = signed #
    // Flags
    // 0 0 H C
    #[allow(dead_code)]
    fn add_sp_n(&mut self, _n: i8) {
        unimplemented!()
    }

    // Increment register nn.
    // n = BC,DE,HL,SP
    fn inc_nn(&mut self, nn: u16) -> u16 {
        nn.wrapping_add(1)
    }

    // Decrement register nn.
    // n = BC,DE,HL,SP
    fn dec_nn(&mut self, nn: u16) -> u16 {
        nn.wrapping_sub(1)
    }

    // Rotate n left. Old bit 7 to Carry flag
    // Flags:
    // Z 0 0 C
    fn rlc_n(&mut self, n: u8) -> u8 {
        let mut res = n << 1;
        if n & 0x80 != 0 {
            res |= 1;
        }
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x80 != 0);
        res
    }

    // Rotate n left through Carry flag.
    // Flags:
    // Z 0 0 C
    fn rl_n(&mut self, n: u8) -> u8 {
        let mut res = n << 1;
        if self.reg.is_flag(C) {
            res |= 0x1;
        }
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x80 != 0);
        res
    }

    // Rotate n right. Old bit 0 to Carry flag
    // Flags:
    // Z 0 0 C
    fn rrc_n(&mut self, n: u8) -> u8 {
        let mut res = n >> 1;
        if n & 0x01 != 0 {
            res |= 0x80;
        }
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x1 != 0);
        res
    }

    // Rotate n right through Carry flag.
    // Flags:
    // Z 0 0 C
    fn rr_n(&mut self, n: u8) -> u8 {
        let mut res = n >> 1;
        if self.reg.is_flag(C) {
            res |= 0x80;
        }
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x1 != 0);
        res
    }

    // Puts bit from register n into Z.
    // b = 0,1,2,3,4,5,6,7
    // n = A,B,C,D,E,H,L,(HL)
    // Flags:
    // Z 0 1 -
    fn bit_b_n(&mut self, b: u8, n: u8) {
        assert!(b <= 7);
        self.reg.set_flag(Z, n & (1 << b) == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }

    fn swap_n(&mut self, n: u8) -> u8 {
        let res = (n << 4) | (n >> 4);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        res
    }

    fn set_b_n(&mut self, b: u8, n: u8) -> u8 {
        n | (1 << b)
    }

    fn res_b_n(&mut self, b: u8, n: u8) -> u8 {
        n & !(1 << b)
    }

    // Shift n right into Carry. MSB set to 0.
    // n = A,B,C,D,E,H,L,(HL)
    // Flags
    // Z 0 0 C
    fn srl_n(&mut self, n: u8) -> u8 {
        let res = n >> 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x1 != 0);
        res
    }

    // Shift n right into Carry. MSB set to 0.
    // n = A,B,C,D,E,H,L,(HL)
    // Flags
    // Z 0 0 C
    fn sra_n(&mut self, n: u8) -> u8 {
        let mut res = n >> 1;
        res |= n & 0x80;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x1 != 0);
        res
    }

    // Shift n left into Carry. LSB of n set to 0
    fn sla_n(&mut self, n: u8) -> u8 {
        let res = n << 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x80 != 0);
        res
    }

    // Pushes present address onto stack.
    // Jump to address $000 + n
    // n = 00,$08,$10,$18,$20,$28,$30,$38
    fn rst_n<C: Cartridge, V: VideoOutput>(&mut self, n: u8, mmu: &mut Mmu<C, V>) {
        self.stack_push(self.reg.pc, mmu);
        self.reg.pc = n as u16;
    }

    // Call Address n if following condition is true:
    // c = NZ, Call if Z flag is reset.
    // c = Z, Call if Z flag is set.
    // c = NC, Call if C flag is reset.
    // c = C, Call if C flag is set.
    fn call_c_n<C: Cartridge, V: VideoOutput>(&mut self, c: bool, mmu: &mut Mmu<C, V>) -> bool {
        let n = self.fetch_word(mmu);
        if c {
            self.stack_push(self.reg.pc, mmu);
            self.reg.pc = n;
        }
        c
    }

    // Push address of next instruction onto the stack and then jump to address n.
    fn call_n<C: Cartridge, V: VideoOutput>(&mut self, mmu: &mut Mmu<C, V>) {
        let n = self.fetch_word(mmu);
        self.stack_push(self.reg.pc, mmu);
        self.reg.pc = n;
    }

    // Jump to address n if following condition is true:
    // c = NZ, Call if Z flag is reset.
    // c = Z, Call if Z flag is set.
    // c = NC, Call if C flag is reset.
    // c = C, Call if C flag is set.
    fn jp_c_n<C: Cartridge, V: VideoOutput>(&mut self, c: bool, mmu: &mut Mmu<C, V>) -> bool {
        let n = self.fetch_word(mmu);
        if c {
            self.reg.pc = n;
        }
        c
    }

    // Add n to current address and jump tp it.
    fn jr_c<C: Cartridge, V: VideoOutput>(&mut self, c: bool, mmu: &mut Mmu<C, V>) -> bool {
        let n = self.fetch_signed(mmu);
        if c {
            let pc = i32::from(self.reg.pc) + i32::from(n);
            self.reg.pc = (pc & 0xffff) as u16;
        }
        c
    }
}

impl Cpu {
    pub fn step<C: Cartridge, V: VideoOutput>(&mut self, mmu: &mut Mmu<C, V>) -> u64 {
        let int = self.int(mmu);
        let c = if int != 0 {
            int
        } else if !self.halt {
            self.exec(mmu)
        } else {
            CYCLES[0x0]
        };
        c * 4
    }

    fn int<C: Cartridge, V: VideoOutput>(&mut self, mmu: &mut Mmu<C, V>) -> u64 {
        let ie = mmu.read(0xffff);
        let if_ = mmu.read(0xff0f);
        let tr = (ie & if_).trailing_zeros() as u8;
        if tr <= 4 {
            self.halt = false;
        }
        if !self.ime || tr > 4 {
            return 0;
        }
        self.int_v([0x40, 0x48, 0x50, 0x58, 0x60][tr as usize], mmu);
        self.ime = false;
        mmu.write(0xff0f, if_ & !(1 << tr));

        #[cfg(feature = "logging")]
        log::info!(target: "cpu", "Disabling all interrupts (IME = 0)");

        4
    }

    fn int_v<C: Cartridge, V: VideoOutput>(&mut self, v: u16, mmu: &mut Mmu<C, V>) {
        #[cfg(feature = "logging")]
        log::info!(target: "cpu", "CALL interrupt vector {:#02x}", v);

        self.stack_push(self.reg.pc, mmu);
        self.reg.pc = v;
    }

    fn exec<C: Cartridge, V: VideoOutput>(&mut self, mmu: &mut Mmu<C, V>) -> u64 {
        let opcode = self.fetch(mmu);
        let mut branch = false;

        match opcode {
            // ADD A,n
            0x80 => self.add_n(self.reg.b),
            0x81 => self.add_n(self.reg.c),
            0x82 => self.add_n(self.reg.d),
            0x83 => self.add_n(self.reg.e),
            0x84 => self.add_n(self.reg.h),
            0x85 => self.add_n(self.reg.l),
            0x86 => self.add_n(mmu.read(self.reg.hl())),
            0x87 => self.add_n(self.reg.a),
            0xc6 => {
                let d8 = self.fetch(mmu);
                self.add_n(d8)
            }
            // ADC A,n
            0x88 => self.adc_n(self.reg.b),
            0x89 => self.adc_n(self.reg.c),
            0x8a => self.adc_n(self.reg.d),
            0x8b => self.adc_n(self.reg.e),
            0x8c => self.adc_n(self.reg.h),
            0x8d => self.adc_n(self.reg.l),
            0x8e => self.adc_n(mmu.read(self.reg.hl())),
            0x8f => self.adc_n(self.reg.a),
            0xce => {
                let d8 = self.fetch(mmu);
                self.adc_n(d8)
            }
            // SUB n
            0x90 => self.sub_n(self.reg.b),
            0x91 => self.sub_n(self.reg.c),
            0x92 => self.sub_n(self.reg.d),
            0x93 => self.sub_n(self.reg.e),
            0x94 => self.sub_n(self.reg.h),
            0x95 => self.sub_n(self.reg.l),
            0x96 => self.sub_n(mmu.read(self.reg.hl())),
            0x97 => self.sub_n(self.reg.a),
            0xd6 => {
                let d8 = self.fetch(mmu);
                self.sub_n(d8)
            }
            // ADC A,n
            0x98 => self.sbc_n(self.reg.b),
            0x99 => self.sbc_n(self.reg.c),
            0x9a => self.sbc_n(self.reg.d),
            0x9b => self.sbc_n(self.reg.e),
            0x9c => self.sbc_n(self.reg.h),
            0x9d => self.sbc_n(self.reg.l),
            0x9e => self.sbc_n(mmu.read(self.reg.hl())),
            0x9f => self.sbc_n(self.reg.a),
            0xde => {
                let d8 = self.fetch(mmu);
                self.sbc_n(d8)
            }
            // AND n
            0xa0 => self.and_n(self.reg.b),
            0xa1 => self.and_n(self.reg.c),
            0xa2 => self.and_n(self.reg.d),
            0xa3 => self.and_n(self.reg.e),
            0xa4 => self.and_n(self.reg.h),
            0xa5 => self.and_n(self.reg.l),
            0xa6 => self.and_n(mmu.read(self.reg.hl())),
            0xa7 => self.and_n(self.reg.a),
            0xe6 => {
                let d8 = self.fetch(mmu);
                self.and_n(d8)
            }
            // XOR n
            0xa8 => self.xor_n(self.reg.b),
            0xa9 => self.xor_n(self.reg.c),
            0xaa => self.xor_n(self.reg.d),
            0xab => self.xor_n(self.reg.e),
            0xac => self.xor_n(self.reg.h),
            0xad => self.xor_n(self.reg.l),
            0xae => self.xor_n(mmu.read(self.reg.hl())),
            0xaf => self.xor_n(self.reg.a),
            0xee => {
                let d8 = self.fetch(mmu);
                self.xor_n(d8)
            }
            // OR n
            0xb0 => self.or_n(self.reg.b),
            0xb1 => self.or_n(self.reg.c),
            0xb2 => self.or_n(self.reg.d),
            0xb3 => self.or_n(self.reg.e),
            0xb4 => self.or_n(self.reg.h),
            0xb5 => self.or_n(self.reg.l),
            0xb6 => self.or_n(mmu.read(self.reg.hl())),
            0xb7 => self.or_n(self.reg.a),
            0xf6 => {
                let d8 = self.fetch(mmu);
                self.or_n(d8)
            }
            // CP n
            0xb8 => self.cp_n(self.reg.b),
            0xb9 => self.cp_n(self.reg.c),
            0xba => self.cp_n(self.reg.d),
            0xbb => self.cp_n(self.reg.e),
            0xbc => self.cp_n(self.reg.h),
            0xbd => self.cp_n(self.reg.l),
            0xbe => self.cp_n(mmu.read(self.reg.hl())),
            0xbf => self.cp_n(self.reg.a),
            0xfe => {
                let d8 = self.fetch(mmu);
                self.cp_n(d8)
            }
            // INC n
            0x04 => self.reg.b = self.inc_n(self.reg.b),
            0x14 => self.reg.d = self.inc_n(self.reg.d),
            0x24 => self.reg.h = self.inc_n(self.reg.h),
            0x34 => {
                let hl = self.reg.hl();
                mmu.write(hl, self.inc_n(mmu.read(hl)));
            }
            0x0c => self.reg.c = self.inc_n(self.reg.c),
            0x1c => self.reg.e = self.inc_n(self.reg.e),
            0x2c => self.reg.l = self.inc_n(self.reg.l),
            0x3c => self.reg.a = self.inc_n(self.reg.a),

            // DEC n
            0x05 => self.reg.b = self.dec_n(self.reg.b),
            0x15 => self.reg.d = self.dec_n(self.reg.d),
            0x25 => self.reg.h = self.dec_n(self.reg.h),
            0x35 => {
                let hl = self.reg.hl();
                mmu.write(hl, self.dec_n(mmu.read(hl)));
            }
            0x0d => self.reg.c = self.dec_n(self.reg.c),
            0x1d => self.reg.e = self.dec_n(self.reg.e),
            0x2d => self.reg.l = self.dec_n(self.reg.l),
            0x3d => self.reg.a = self.dec_n(self.reg.a),

            // INC nn
            0x03 => {
                let r = self.inc_nn(self.reg.bc());
                self.reg.set_bc(r)
            }
            0x13 => {
                let r = self.inc_nn(self.reg.de());
                self.reg.set_de(r)
            }
            0x23 => {
                let r = self.inc_nn(self.reg.hl());
                self.reg.set_hl(r)
            }
            0x33 => self.reg.sp = self.inc_nn(self.reg.sp),
            // DEC nn
            0x0b => {
                let r = self.dec_nn(self.reg.bc());
                self.reg.set_bc(r)
            }
            0x1b => {
                let r = self.dec_nn(self.reg.de());
                self.reg.set_de(r)
            }
            0x2b => {
                let r = self.dec_nn(self.reg.hl());
                self.reg.set_hl(r)
            }
            0x3b => self.reg.sp = self.dec_nn(self.reg.sp),
            // ADD HL,nn
            0x09 => self.add_hl_nn(self.reg.bc()),
            0x19 => self.add_hl_nn(self.reg.de()),
            0x29 => self.add_hl_nn(self.reg.hl()),
            0x39 => self.add_hl_nn(self.reg.sp),

            // ADD SP,r8
            0xe8 => {
                let a = self.reg.sp;
                let b = i16::from(self.fetch_signed(mmu)) as u16;
                self.reg.set_flag(C, (a & 0xff) + (b & 0xff) > 0xff);
                self.reg.set_flag(H, (a & 0xf) + (b & 0xf) > 0xf);
                self.reg.set_flag(N, false);
                self.reg.set_flag(Z, false);
                self.reg.sp = a.wrapping_add(b);
            }

            // SCF
            0x37 => {
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
                self.reg.set_flag(C, true);
            }
            // CCF
            0x3f => {
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
                self.reg.set_flag(C, !self.reg.is_flag(C));
            }
            0x27 => {
                let mut a = self.reg.a;
                let mut adjust = if self.reg.is_flag(C) { 0x60 } else { 0x00 };
                if self.reg.is_flag(H) {
                    adjust |= 0x06;
                };
                if !self.reg.is_flag(N) {
                    if a & 0x0f > 0x09 {
                        adjust |= 0x06;
                    };
                    if a > 0x99 {
                        adjust |= 0x60;
                    };
                    a = a.wrapping_add(adjust);
                } else {
                    a = a.wrapping_sub(adjust);
                }
                self.reg.set_flag(C, adjust >= 0x60);
                self.reg.set_flag(H, false);
                self.reg.set_flag(Z, a == 0x00);
                self.reg.a = a;
            }
            // CPL
            0x2f => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(N, true);
                self.reg.set_flag(H, true);
            }

            // FIXME CONFLICT
            // according to https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
            // Z is reset, but according to http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
            // Z depends on the result
            //
            // UPDATE: opcode table is right
            0x07 => {
                self.reg.a = self.rlc_n(self.reg.a);
                self.reg.set_flag(Z, false); // 09-op r,r.gb
            }
            0x17 => {
                self.reg.a = self.rl_n(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            0x0f => {
                self.reg.a = self.rrc_n(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            0x1f => {
                self.reg.a = self.rr_n(self.reg.a);
                self.reg.set_flag(Z, false);
            }

            // LD B,n
            0x40 => self.reg.b = self.reg.b,
            0x41 => self.reg.b = self.reg.c,
            0x42 => self.reg.b = self.reg.d,
            0x43 => self.reg.b = self.reg.e,
            0x44 => self.reg.b = self.reg.h,
            0x45 => self.reg.b = self.reg.l,
            0x46 => self.reg.b = mmu.read(self.reg.hl()),
            0x06 => self.reg.b = self.fetch(mmu),
            0x47 => self.reg.b = self.reg.a,
            // LD C,n
            0x48 => self.reg.c = self.reg.b,
            0x49 => self.reg.c = self.reg.c,
            0x4a => self.reg.c = self.reg.d,
            0x4b => self.reg.c = self.reg.e,
            0x4c => self.reg.c = self.reg.h,
            0x4d => self.reg.c = self.reg.l,
            0x4e => self.reg.c = mmu.read(self.reg.hl()),
            0x0e => self.reg.c = self.fetch(mmu),
            0x4f => self.reg.c = self.reg.a,
            // LD D,n
            0x50 => self.reg.d = self.reg.b,
            0x51 => self.reg.d = self.reg.c,
            0x52 => self.reg.d = self.reg.d,
            0x53 => self.reg.d = self.reg.e,
            0x54 => self.reg.d = self.reg.h,
            0x55 => self.reg.d = self.reg.l,
            0x56 => self.reg.d = mmu.read(self.reg.hl()),
            0x16 => self.reg.d = self.fetch(mmu),
            0x57 => self.reg.d = self.reg.a,
            // LD E,n
            0x58 => self.reg.e = self.reg.b,
            0x59 => self.reg.e = self.reg.c,
            0x5a => self.reg.e = self.reg.d,
            0x5b => self.reg.e = self.reg.e,
            0x5c => self.reg.e = self.reg.h,
            0x5d => self.reg.e = self.reg.l,
            0x5e => self.reg.e = mmu.read(self.reg.hl()),
            0x1e => self.reg.e = self.fetch(mmu),
            0x5f => self.reg.e = self.reg.a,
            // LD H,n
            0x60 => self.reg.h = self.reg.b,
            0x61 => self.reg.h = self.reg.c,
            0x62 => self.reg.h = self.reg.d,
            0x63 => self.reg.h = self.reg.e,
            0x64 => self.reg.h = self.reg.h,
            0x65 => self.reg.h = self.reg.l,
            0x66 => self.reg.h = mmu.read(self.reg.hl()),
            0x26 => self.reg.h = self.fetch(mmu),
            0x67 => self.reg.h = self.reg.a,
            // LD L,n
            0x68 => self.reg.l = self.reg.b,
            0x69 => self.reg.l = self.reg.c,
            0x6a => self.reg.l = self.reg.d,
            0x6b => self.reg.l = self.reg.e,
            0x6c => self.reg.l = self.reg.h,
            0x6d => self.reg.l = self.reg.l,
            0x6e => self.reg.l = mmu.read(self.reg.hl()),
            0x2e => self.reg.l = self.fetch(mmu),
            0x6f => self.reg.l = self.reg.a,
            // LD (HL),n
            0x70 => mmu.write(self.reg.hl(), self.reg.b),
            0x71 => mmu.write(self.reg.hl(), self.reg.c),
            0x72 => mmu.write(self.reg.hl(), self.reg.d),
            0x73 => mmu.write(self.reg.hl(), self.reg.e),
            0x74 => mmu.write(self.reg.hl(), self.reg.h),
            0x75 => mmu.write(self.reg.hl(), self.reg.l),
            0x36 => mmu.write(self.reg.hl(), self.fetch(mmu)),
            0x77 => mmu.write(self.reg.hl(), self.reg.a),
            // LD A,n
            0x78 => self.reg.a = self.reg.b,
            0x79 => self.reg.a = self.reg.c,
            0x7a => self.reg.a = self.reg.d,
            0x7b => self.reg.a = self.reg.e,
            0x7c => self.reg.a = self.reg.h,
            0x7d => self.reg.a = self.reg.l,
            0x7e => self.reg.a = mmu.read(self.reg.hl()),
            0x3e => self.reg.a = self.fetch(mmu),
            0x7f => self.reg.a = self.reg.a,
            // LD (a16),SP
            0x08 => {
                let a16 = self.fetch_word(mmu);
                let sp = self.reg.sp;
                mmu.write(a16, (sp & 0xff) as u8);
                mmu.write(a16 + 1, ((sp >> 8) & 0xff) as u8);
            }
            // LD nn,d16
            0x01 => {
                let d = self.fetch_word(mmu);
                self.reg.set_bc(d)
            }
            0x11 => {
                let d = self.fetch_word(mmu);
                self.reg.set_de(d)
            }
            0x21 => {
                let d = self.fetch_word(mmu);
                self.reg.set_hl(d)
            }
            0x31 => self.reg.sp = self.fetch_word(mmu),
            // LD HL,SP+r8
            0xf8 => {
                let a = self.reg.sp;
                let b = i16::from(self.fetch_signed(mmu)) as u16;
                self.reg.set_flag(C, (a & 0x00ff) + (b & 0x00ff) > 0x00ff);
                self.reg.set_flag(H, (a & 0x000f) + (b & 0x000f) > 0x000f);
                self.reg.set_flag(N, false);
                self.reg.set_flag(Z, false);
                self.reg.set_hl(a.wrapping_add(b));
            }
            // LD SP,HL
            0xf9 => self.reg.sp = self.reg.hl(),
            // LD (nn),A
            0x02 => mmu.write(self.reg.bc(), self.reg.a),
            0x12 => mmu.write(self.reg.de(), self.reg.a),
            0x22 => {
                let hl = self.reg.hl();
                mmu.write(hl, self.reg.a);
                self.reg.set_hl(hl.wrapping_add(1));
            }
            0x32 => {
                let hl = self.reg.hl();
                mmu.write(hl, self.reg.a);
                self.reg.set_hl(hl.wrapping_sub(1));
            }
            // LD A,(nn)
            0x0a => self.reg.a = mmu.read(self.reg.bc()),
            0x1a => self.reg.a = mmu.read(self.reg.de()),
            0x2a => {
                self.reg.a = {
                    let hl = self.reg.hl();
                    let d = mmu.read(hl);
                    self.reg.set_hl(hl.wrapping_add(1));
                    d
                }
            }
            0x3a => {
                self.reg.a = {
                    let hl = self.reg.hl();
                    let d = mmu.read(hl);
                    self.reg.set_hl(hl.wrapping_sub(1));
                    d
                }
            }

            0xe2 => mmu.write(0xff00 + u16::from(self.reg.c), self.reg.a),
            0xf2 => self.reg.a = mmu.read(0xff00 + u16::from(self.reg.c)),

            0xe0 => {
                let a8 = self.fetch(mmu) as u16;
                mmu.write(0xff00 + a8, self.reg.a);
            }
            0xf0 => {
                let a8 = self.fetch(mmu) as u16;
                self.reg.a = mmu.read(0xff00 + a8);
            }

            0xea => mmu.write(self.fetch_word(mmu), self.reg.a),
            0xfa => {
                let a16 = self.fetch_word(mmu);
                self.reg.a = mmu.read(a16);
            }

            // POP nn
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let r = self.stack_pop(mmu);
                match opcode {
                    0xc1 => self.reg.set_bc(r),
                    0xd1 => self.reg.set_de(r),
                    0xe1 => self.reg.set_hl(r),
                    0xf1 => self.reg.set_af(r & 0xfff0),
                    // BUG
                    _ => panic!(),
                }
            }
            // PUSH nn
            0xc5 => self.stack_push(self.reg.bc(), mmu),
            0xd5 => self.stack_push(self.reg.de(), mmu),
            0xe5 => self.stack_push(self.reg.hl(), mmu),
            0xf5 => self.stack_push(self.reg.af(), mmu),

            // RET cc
            0xc0 => {
                if !self.reg.is_flag(Z) {
                    self.reg.pc = self.stack_pop(mmu);
                }
            }
            0xd0 => {
                if !self.reg.is_flag(C) {
                    self.reg.pc = self.stack_pop(mmu);
                }
            }
            0xc8 => {
                if self.reg.is_flag(Z) {
                    self.reg.pc = self.stack_pop(mmu);
                }
            }
            0xd8 => {
                if self.reg.is_flag(C) {
                    self.reg.pc = self.stack_pop(mmu);
                }
            }
            // RET
            0xc9 => self.reg.pc = self.stack_pop(mmu),
            // RETI
            0xd9 => {
                #[cfg(feature = "logging")]
                log::info!(target: "cpu", "RETI");

                self.ime = true;
                self.reg.pc = self.stack_pop(mmu);
            }

            0xc7 => self.rst_n(0x00, mmu),
            0xd7 => self.rst_n(0x10, mmu),
            0xe7 => self.rst_n(0x20, mmu),
            0xf7 => self.rst_n(0x30, mmu),
            0xcf => self.rst_n(0x08, mmu),
            0xdf => self.rst_n(0x18, mmu),
            0xef => self.rst_n(0x28, mmu),
            0xff => self.rst_n(0x38, mmu),

            0xc4 => branch = self.call_c_n(!self.reg.is_flag(Z), mmu),
            0xd4 => branch = self.call_c_n(!self.reg.is_flag(C), mmu),
            0xcc => branch = self.call_c_n(self.reg.is_flag(Z), mmu),
            0xdc => branch = self.call_c_n(self.reg.is_flag(C), mmu),
            0xcd => self.call_n(mmu),

            0xc2 => branch = self.jp_c_n(!self.reg.is_flag(Z), mmu),
            0xd2 => branch = self.jp_c_n(!self.reg.is_flag(C), mmu),
            0xca => branch = self.jp_c_n(self.reg.is_flag(Z), mmu),
            0xda => branch = self.jp_c_n(self.reg.is_flag(C), mmu),
            0xc3 => self.reg.pc = self.fetch_word(mmu),
            0xe9 => {
                // The pdf was ambiguous. Verified with other emulators:
                // - https://github.com/taisel/GameBoy-Online/blob/master/js/GameBoyCore.js#L2086
                // - https://github.com/HFO4/gameboy.live/blob/master/gb/opcodes.go#L2103
                self.reg.pc = self.reg.hl()
            }

            0x20 => branch = self.jr_c(!self.reg.is_flag(Z), mmu),
            0x30 => branch = self.jr_c(!self.reg.is_flag(C), mmu),
            0x28 => branch = self.jr_c(self.reg.is_flag(Z), mmu),
            0x38 => branch = self.jr_c(self.reg.is_flag(C), mmu),
            0x18 => {
                self.jr_c(true, mmu);
            }

            // Misc/control instructions
            0x00 => {} // NOP
            //0x10 => unimplemented!("0x10 - STOP 0 - not implemented"), // STOP 0
            0x10 => {}
            0x76 => {
                #[cfg(feature = "logging")]
                log::info!(target: "cpu", "HALT");

                self.halt = true
            }
            0xf3 => {
                #[cfg(feature = "logging")]
                log::info!(target: "cpu", "IME = 0");

                self.ime = false
            }
            0xfb => {
                #[cfg(feature = "logging")]
                log::info!(target: "cpu", "IME = 1");

                self.ime = true
            }
            0xcb => {
                let cb = self.fetch(mmu);
                match cb {
                    // RLC n
                    0x00 => self.reg.b = self.rlc_n(self.reg.b),
                    0x01 => self.reg.c = self.rlc_n(self.reg.c),
                    0x02 => self.reg.d = self.rlc_n(self.reg.d),
                    0x03 => self.reg.e = self.rlc_n(self.reg.e),
                    0x04 => self.reg.h = self.rlc_n(self.reg.h),
                    0x05 => self.reg.l = self.rlc_n(self.reg.l),
                    0x06 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.rlc_n(mmu.read(hl)))
                    }
                    0x07 => self.reg.a = self.rlc_n(self.reg.a),

                    // RRC n
                    0x08 => self.reg.b = self.rrc_n(self.reg.b),
                    0x09 => self.reg.c = self.rrc_n(self.reg.c),
                    0x0a => self.reg.d = self.rrc_n(self.reg.d),
                    0x0b => self.reg.e = self.rrc_n(self.reg.e),
                    0x0c => self.reg.h = self.rrc_n(self.reg.h),
                    0x0d => self.reg.l = self.rrc_n(self.reg.l),
                    0x0e => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.rrc_n(mmu.read(hl)))
                    }
                    0x0f => self.reg.a = self.rrc_n(self.reg.a),

                    // RL n
                    0x10 => self.reg.b = self.rl_n(self.reg.b),
                    0x11 => self.reg.c = self.rl_n(self.reg.c),
                    0x12 => self.reg.d = self.rl_n(self.reg.d),
                    0x13 => self.reg.e = self.rl_n(self.reg.e),
                    0x14 => self.reg.h = self.rl_n(self.reg.h),
                    0x15 => self.reg.l = self.rl_n(self.reg.l),
                    0x16 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.rl_n(mmu.read(hl)))
                    }
                    0x17 => self.reg.a = self.rl_n(self.reg.a),

                    // RR n
                    0x18 => self.reg.b = self.rr_n(self.reg.b),
                    0x19 => self.reg.c = self.rr_n(self.reg.c),
                    0x1a => self.reg.d = self.rr_n(self.reg.d),
                    0x1b => self.reg.e = self.rr_n(self.reg.e),
                    0x1c => self.reg.h = self.rr_n(self.reg.h),
                    0x1d => self.reg.l = self.rr_n(self.reg.l),
                    0x1e => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.rr_n(mmu.read(hl)))
                    }
                    0x1f => self.reg.a = self.rr_n(self.reg.a),

                    // SWAP n
                    0x30 => self.reg.b = self.swap_n(self.reg.b),
                    0x31 => self.reg.c = self.swap_n(self.reg.c),
                    0x32 => self.reg.d = self.swap_n(self.reg.d),
                    0x33 => self.reg.e = self.swap_n(self.reg.e),
                    0x34 => self.reg.h = self.swap_n(self.reg.h),
                    0x35 => self.reg.l = self.swap_n(self.reg.l),
                    0x36 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.swap_n(mmu.read(hl)))
                    }
                    0x37 => self.reg.a = self.swap_n(self.reg.a),

                    // BIT 0,n
                    0x40 => self.bit_b_n(0, self.reg.b),
                    0x41 => self.bit_b_n(0, self.reg.c),
                    0x42 => self.bit_b_n(0, self.reg.d),
                    0x43 => self.bit_b_n(0, self.reg.e),
                    0x44 => self.bit_b_n(0, self.reg.h),
                    0x45 => self.bit_b_n(0, self.reg.l),
                    0x46 => self.bit_b_n(0, mmu.read(self.reg.hl())),
                    0x47 => self.bit_b_n(0, self.reg.a),

                    // BIT 1,n
                    0x48 => self.bit_b_n(1, self.reg.b),
                    0x49 => self.bit_b_n(1, self.reg.c),
                    0x4a => self.bit_b_n(1, self.reg.d),
                    0x4b => self.bit_b_n(1, self.reg.e),
                    0x4c => self.bit_b_n(1, self.reg.h),
                    0x4d => self.bit_b_n(1, self.reg.l),
                    0x4e => self.bit_b_n(1, mmu.read(self.reg.hl())),
                    0x4f => self.bit_b_n(1, self.reg.a),

                    // BIT 2,n
                    0x50 => self.bit_b_n(2, self.reg.b),
                    0x51 => self.bit_b_n(2, self.reg.c),
                    0x52 => self.bit_b_n(2, self.reg.d),
                    0x53 => self.bit_b_n(2, self.reg.e),
                    0x54 => self.bit_b_n(2, self.reg.h),
                    0x55 => self.bit_b_n(2, self.reg.l),
                    0x56 => self.bit_b_n(2, mmu.read(self.reg.hl())),
                    0x57 => self.bit_b_n(2, self.reg.a),

                    // BIT 3,n
                    0x58 => self.bit_b_n(3, self.reg.b),
                    0x59 => self.bit_b_n(3, self.reg.c),
                    0x5a => self.bit_b_n(3, self.reg.d),
                    0x5b => self.bit_b_n(3, self.reg.e),
                    0x5c => self.bit_b_n(3, self.reg.h),
                    0x5d => self.bit_b_n(3, self.reg.l),
                    0x5e => self.bit_b_n(3, mmu.read(self.reg.hl())),
                    0x5f => self.bit_b_n(3, self.reg.a),

                    // BIT 4,n
                    0x60 => self.bit_b_n(4, self.reg.b),
                    0x61 => self.bit_b_n(4, self.reg.c),
                    0x62 => self.bit_b_n(4, self.reg.d),
                    0x63 => self.bit_b_n(4, self.reg.e),
                    0x64 => self.bit_b_n(4, self.reg.h),
                    0x65 => self.bit_b_n(4, self.reg.l),
                    0x66 => self.bit_b_n(4, mmu.read(self.reg.hl())),
                    0x67 => self.bit_b_n(4, self.reg.a),

                    // BIT 5,n
                    0x68 => self.bit_b_n(5, self.reg.b),
                    0x69 => self.bit_b_n(5, self.reg.c),
                    0x6a => self.bit_b_n(5, self.reg.d),
                    0x6b => self.bit_b_n(5, self.reg.e),
                    0x6c => self.bit_b_n(5, self.reg.h),
                    0x6d => self.bit_b_n(5, self.reg.l),
                    0x6e => self.bit_b_n(5, mmu.read(self.reg.hl())),
                    0x6f => self.bit_b_n(5, self.reg.a),

                    // BIT 6,n
                    0x70 => self.bit_b_n(6, self.reg.b),
                    0x71 => self.bit_b_n(6, self.reg.c),
                    0x72 => self.bit_b_n(6, self.reg.d),
                    0x73 => self.bit_b_n(6, self.reg.e),
                    0x74 => self.bit_b_n(6, self.reg.h),
                    0x75 => self.bit_b_n(6, self.reg.l),
                    0x76 => self.bit_b_n(6, mmu.read(self.reg.hl())),
                    0x77 => self.bit_b_n(6, self.reg.a),

                    // BIT 7,n
                    0x78 => self.bit_b_n(7, self.reg.b),
                    0x79 => self.bit_b_n(7, self.reg.c),
                    0x7a => self.bit_b_n(7, self.reg.d),
                    0x7b => self.bit_b_n(7, self.reg.e),
                    0x7c => self.bit_b_n(7, self.reg.h),
                    0x7d => self.bit_b_n(7, self.reg.l),
                    0x7e => self.bit_b_n(7, mmu.read(self.reg.hl())),
                    0x7f => self.bit_b_n(7, self.reg.a),

                    // RES 0,n
                    0x80 => self.reg.b = self.res_b_n(0, self.reg.b),
                    0x81 => self.reg.c = self.res_b_n(0, self.reg.c),
                    0x82 => self.reg.d = self.res_b_n(0, self.reg.d),
                    0x83 => self.reg.e = self.res_b_n(0, self.reg.e),
                    0x84 => self.reg.h = self.res_b_n(0, self.reg.h),
                    0x85 => self.reg.l = self.res_b_n(0, self.reg.l),
                    0x86 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(0, mmu.read(self.reg.hl())))
                    }
                    0x87 => self.reg.a = self.res_b_n(0, self.reg.a),

                    // RES 1,n
                    0x88 => self.reg.b = self.res_b_n(1, self.reg.b),
                    0x89 => self.reg.c = self.res_b_n(1, self.reg.c),
                    0x8a => self.reg.d = self.res_b_n(1, self.reg.d),
                    0x8b => self.reg.e = self.res_b_n(1, self.reg.e),
                    0x8c => self.reg.h = self.res_b_n(1, self.reg.h),
                    0x8d => self.reg.l = self.res_b_n(1, self.reg.l),
                    0x8e => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(1, mmu.read(self.reg.hl())))
                    }
                    0x8f => self.reg.a = self.res_b_n(1, self.reg.a),

                    // RES 2,n
                    0x90 => self.reg.b = self.res_b_n(2, self.reg.b),
                    0x91 => self.reg.c = self.res_b_n(2, self.reg.c),
                    0x92 => self.reg.d = self.res_b_n(2, self.reg.d),
                    0x93 => self.reg.e = self.res_b_n(2, self.reg.e),
                    0x94 => self.reg.h = self.res_b_n(2, self.reg.h),
                    0x95 => self.reg.l = self.res_b_n(2, self.reg.l),
                    0x96 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(2, mmu.read(self.reg.hl())))
                    }
                    0x97 => self.reg.a = self.res_b_n(2, self.reg.a),

                    // RES 3,n
                    0x98 => self.reg.b = self.res_b_n(3, self.reg.b),
                    0x99 => self.reg.c = self.res_b_n(3, self.reg.c),
                    0x9a => self.reg.d = self.res_b_n(3, self.reg.d),
                    0x9b => self.reg.e = self.res_b_n(3, self.reg.e),
                    0x9c => self.reg.h = self.res_b_n(3, self.reg.h),
                    0x9d => self.reg.l = self.res_b_n(3, self.reg.l),
                    0x9e => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(3, mmu.read(self.reg.hl())))
                    }
                    0x9f => self.reg.a = self.res_b_n(3, self.reg.a),

                    // RES 4,n
                    0xa0 => self.reg.b = self.res_b_n(4, self.reg.b),
                    0xa1 => self.reg.c = self.res_b_n(4, self.reg.c),
                    0xa2 => self.reg.d = self.res_b_n(4, self.reg.d),
                    0xa3 => self.reg.e = self.res_b_n(4, self.reg.e),
                    0xa4 => self.reg.h = self.res_b_n(4, self.reg.h),
                    0xa5 => self.reg.l = self.res_b_n(4, self.reg.l),
                    0xa6 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(4, mmu.read(self.reg.hl())))
                    }
                    0xa7 => self.reg.a = self.res_b_n(4, self.reg.a),

                    // RES 5,n
                    0xa8 => self.reg.b = self.res_b_n(5, self.reg.b),
                    0xa9 => self.reg.c = self.res_b_n(5, self.reg.c),
                    0xaa => self.reg.d = self.res_b_n(5, self.reg.d),
                    0xab => self.reg.e = self.res_b_n(5, self.reg.e),
                    0xac => self.reg.h = self.res_b_n(5, self.reg.h),
                    0xad => self.reg.l = self.res_b_n(5, self.reg.l),
                    0xae => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(5, mmu.read(self.reg.hl())))
                    }
                    0xaf => self.reg.a = self.res_b_n(5, self.reg.a),

                    // RES 6,n
                    0xb0 => self.reg.b = self.res_b_n(6, self.reg.b),
                    0xb1 => self.reg.c = self.res_b_n(6, self.reg.c),
                    0xb2 => self.reg.d = self.res_b_n(6, self.reg.d),
                    0xb3 => self.reg.e = self.res_b_n(6, self.reg.e),
                    0xb4 => self.reg.h = self.res_b_n(6, self.reg.h),
                    0xb5 => self.reg.l = self.res_b_n(6, self.reg.l),
                    0xb6 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(6, mmu.read(self.reg.hl())))
                    }
                    0xb7 => self.reg.a = self.res_b_n(6, self.reg.a),

                    // RES 7,n
                    0xb8 => self.reg.b = self.res_b_n(7, self.reg.b),
                    0xb9 => self.reg.c = self.res_b_n(7, self.reg.c),
                    0xba => self.reg.d = self.res_b_n(7, self.reg.d),
                    0xbb => self.reg.e = self.res_b_n(7, self.reg.e),
                    0xbc => self.reg.h = self.res_b_n(7, self.reg.h),
                    0xbd => self.reg.l = self.res_b_n(7, self.reg.l),
                    0xbe => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.res_b_n(7, mmu.read(self.reg.hl())))
                    }
                    0xbf => self.reg.a = self.res_b_n(7, self.reg.a),

                    // SET 0,n
                    0xc0 => self.reg.b = self.set_b_n(0, self.reg.b),
                    0xc1 => self.reg.c = self.set_b_n(0, self.reg.c),
                    0xc2 => self.reg.d = self.set_b_n(0, self.reg.d),
                    0xc3 => self.reg.e = self.set_b_n(0, self.reg.e),
                    0xc4 => self.reg.h = self.set_b_n(0, self.reg.h),
                    0xc5 => self.reg.l = self.set_b_n(0, self.reg.l),
                    0xc6 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(0, mmu.read(self.reg.hl())))
                    }
                    0xc7 => self.reg.a = self.set_b_n(0, self.reg.a),

                    // SET 1,n
                    0xc8 => self.reg.b = self.set_b_n(1, self.reg.b),
                    0xc9 => self.reg.c = self.set_b_n(1, self.reg.c),
                    0xca => self.reg.d = self.set_b_n(1, self.reg.d),
                    0xcb => self.reg.e = self.set_b_n(1, self.reg.e),
                    0xcc => self.reg.h = self.set_b_n(1, self.reg.h),
                    0xcd => self.reg.l = self.set_b_n(1, self.reg.l),
                    0xce => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(1, mmu.read(self.reg.hl())))
                    }
                    0xcf => self.reg.a = self.set_b_n(1, self.reg.a),

                    // SET 2,n
                    0xd0 => self.reg.b = self.set_b_n(2, self.reg.b),
                    0xd1 => self.reg.c = self.set_b_n(2, self.reg.c),
                    0xd2 => self.reg.d = self.set_b_n(2, self.reg.d),
                    0xd3 => self.reg.e = self.set_b_n(2, self.reg.e),
                    0xd4 => self.reg.h = self.set_b_n(2, self.reg.h),
                    0xd5 => self.reg.l = self.set_b_n(2, self.reg.l),
                    0xd6 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(2, mmu.read(self.reg.hl())))
                    }
                    0xd7 => self.reg.a = self.set_b_n(2, self.reg.a),

                    // SET 3,n
                    0xd8 => self.reg.b = self.set_b_n(3, self.reg.b),
                    0xd9 => self.reg.c = self.set_b_n(3, self.reg.c),
                    0xda => self.reg.d = self.set_b_n(3, self.reg.d),
                    0xdb => self.reg.e = self.set_b_n(3, self.reg.e),
                    0xdc => self.reg.h = self.set_b_n(3, self.reg.h),
                    0xdd => self.reg.l = self.set_b_n(3, self.reg.l),
                    0xde => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(3, mmu.read(self.reg.hl())))
                    }
                    0xdf => self.reg.a = self.set_b_n(3, self.reg.a),

                    // SET 4,n
                    0xe0 => self.reg.b = self.set_b_n(4, self.reg.b),
                    0xe1 => self.reg.c = self.set_b_n(4, self.reg.c),
                    0xe2 => self.reg.d = self.set_b_n(4, self.reg.d),
                    0xe3 => self.reg.e = self.set_b_n(4, self.reg.e),
                    0xe4 => self.reg.h = self.set_b_n(4, self.reg.h),
                    0xe5 => self.reg.l = self.set_b_n(4, self.reg.l),
                    0xe6 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(4, mmu.read(self.reg.hl())))
                    }
                    0xe7 => self.reg.a = self.set_b_n(4, self.reg.a),

                    // SET 5,n
                    0xe8 => self.reg.b = self.set_b_n(5, self.reg.b),
                    0xe9 => self.reg.c = self.set_b_n(5, self.reg.c),
                    0xea => self.reg.d = self.set_b_n(5, self.reg.d),
                    0xeb => self.reg.e = self.set_b_n(5, self.reg.e),
                    0xec => self.reg.h = self.set_b_n(5, self.reg.h),
                    0xed => self.reg.l = self.set_b_n(5, self.reg.l),
                    0xee => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(5, mmu.read(self.reg.hl())))
                    }
                    0xef => self.reg.a = self.set_b_n(5, self.reg.a),

                    // SET 6,n
                    0xf0 => self.reg.b = self.set_b_n(6, self.reg.b),
                    0xf1 => self.reg.c = self.set_b_n(6, self.reg.c),
                    0xf2 => self.reg.d = self.set_b_n(6, self.reg.d),
                    0xf3 => self.reg.e = self.set_b_n(6, self.reg.e),
                    0xf4 => self.reg.h = self.set_b_n(6, self.reg.h),
                    0xf5 => self.reg.l = self.set_b_n(6, self.reg.l),
                    0xf6 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(6, mmu.read(self.reg.hl())))
                    }
                    0xf7 => self.reg.a = self.set_b_n(6, self.reg.a),

                    // SET 7,n
                    0xf8 => self.reg.b = self.set_b_n(7, self.reg.b),
                    0xf9 => self.reg.c = self.set_b_n(7, self.reg.c),
                    0xfa => self.reg.d = self.set_b_n(7, self.reg.d),
                    0xfb => self.reg.e = self.set_b_n(7, self.reg.e),
                    0xfc => self.reg.h = self.set_b_n(7, self.reg.h),
                    0xfd => self.reg.l = self.set_b_n(7, self.reg.l),
                    0xfe => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.set_b_n(7, mmu.read(self.reg.hl())))
                    }
                    0xff => self.reg.a = self.set_b_n(7, self.reg.a),

                    // SRL n
                    0x38 => self.reg.b = self.srl_n(self.reg.b),
                    0x39 => self.reg.c = self.srl_n(self.reg.c),
                    0x3a => self.reg.d = self.srl_n(self.reg.d),
                    0x3b => self.reg.e = self.srl_n(self.reg.e),
                    0x3c => self.reg.h = self.srl_n(self.reg.h),
                    0x3d => self.reg.l = self.srl_n(self.reg.l),
                    0x3e => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.srl_n(mmu.read(self.reg.hl())))
                    }
                    0x3f => self.reg.a = self.srl_n(self.reg.a),

                    // SLA n
                    0x20 => self.reg.b = self.sla_n(self.reg.b),
                    0x21 => self.reg.c = self.sla_n(self.reg.c),
                    0x22 => self.reg.d = self.sla_n(self.reg.d),
                    0x23 => self.reg.e = self.sla_n(self.reg.e),
                    0x24 => self.reg.h = self.sla_n(self.reg.h),
                    0x25 => self.reg.l = self.sla_n(self.reg.l),
                    0x26 => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.sla_n(mmu.read(self.reg.hl())))
                    }
                    0x27 => self.reg.a = self.sla_n(self.reg.a),

                    // SRA n
                    0x28 => self.reg.b = self.sra_n(self.reg.b),
                    0x29 => self.reg.c = self.sra_n(self.reg.c),
                    0x2a => self.reg.d = self.sra_n(self.reg.d),
                    0x2b => self.reg.e = self.sra_n(self.reg.e),
                    0x2c => self.reg.h = self.sra_n(self.reg.h),
                    0x2d => self.reg.l = self.sra_n(self.reg.l),
                    0x2e => {
                        let hl = self.reg.hl();
                        mmu.write(hl, self.sra_n(mmu.read(self.reg.hl())))
                    }
                    0x2f => self.reg.a = self.sra_n(self.reg.a),
                }

                return CB_CYCLES[cb as usize];
            }

            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb..=0xed | 0xf4 | 0xfc | 0xfd => {
                #[cfg(feature = "logging")]
                log::error!(target: "cpu", "Reached undefined OPCODE {:#02x}", opcode);

                panic!("Undefined opcode = 0x{:02x}", opcode)
            }
        }

        // conditional instructions
        #[rustfmt::skip]
        let cycles = match opcode {
            // JR
            0x20 | 0x30 | 0x28 | 0x38 => if branch { 3 } else { 1 }
            // RET
            0xc0 | 0xd0 | 0xc8 | 0xd8 => if branch { 5 } else { 2 }
            // JP
            0xc2 | 0xd2 | 0xca | 0xda => if branch { 4 } else { 3 }
            // CALL
            0xc4 | 0xd4 | 0xcc | 0xdc => if branch { 6 } else { 3 }
            _ => CYCLES[opcode as usize],
        };
        cycles.max(1)
    }
}
