use crate::{
    device::Device,
    mmu::Mmu,
    registers::{Flag::*, Registers},
};

pub struct Cpu {
    reg: Registers,
    ime: bool,
    halt: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            ime: false,
            halt: false,
        }
    }

    pub fn reg(&self) -> &Registers {
        &self.reg
    }

    fn fetch(&mut self, mmu: &Mmu) -> u8 {
        let b = mmu.read(self.reg.pc);
        self.reg.pc += 1;
        b
    }

    fn fetch_word(&mut self, mmu: &Mmu) -> u16 {
        let w = mmu.read_word(self.reg.pc);
        self.reg.pc += 2;
        w
    }

    fn fetch_signed(&mut self, mmu: &Mmu) -> i8 {
        let n: i8 = unsafe { std::mem::transmute(self.fetch(mmu)) };
        n
    }

    // Pushes word into the stack
    // Decrements SP by 2
    fn stack_push(&mut self, nn: u16, mmu: &mut Mmu) {
        self.reg.sp -= 1;
        mmu.write_word(self.reg.sp, nn);
        self.reg.sp -= 1;
    }

    // Pops word from the stack
    // Increments SP by 2
    fn stack_pop(&mut self, mmu: &Mmu) -> u16 {
        self.reg.sp += 1;
        let nn = mmu.read_word(self.reg.sp);
        self.reg.sp += 1;
        nn
    }

    // Add n to A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 H C
    fn alu_add_n(&mut self, n: u8) {
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
    fn alu_adc_n(&mut self, n: u8) {
        let mut res = u16::from(self.reg.a) + u16::from(n);
        if self.reg.is_flag(C) {
            res += 1;
        }
        self.reg.set_flag(Z, res.trailing_zeros() >= 8);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (self.reg.a & 0xf) + (n & 0xf) > 0xf);
        self.reg.set_flag(C, res > 0xff);
        self.reg.a = (res & 0xff) as u8;
    }

    // Subtract n from A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn alu_sub_n(&mut self, n: u8) {
        let res = self.reg.a.wrapping_sub(n);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        // reference: https://github.com/CTurt/Cinoop/blob/master/source/cpu.c#L589
        self.reg.set_flag(H, n & 0xf > self.reg.a & 0xf);
        self.reg.set_flag(C, n > self.reg.a);
        self.reg.a = res;
    }
    // Subtract n + Carry flag from A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn alu_sbc_n(&mut self, n: u8) {
        let c = if self.reg.is_flag(C) { 1 } else { 0 };
        let res = self.reg.a.wrapping_sub(n).wrapping_sub(c);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        // ref: https://github.com/CTurt/Cinoop/blob/master/source/cpu.c#L572
        self.reg.set_flag(H, (n & 0xf) + c > self.reg.a & 0xf);
        self.reg
            .set_flag(C, u16::from(n) + u16::from(c) > u16::from(self.reg.a));
        self.reg.a = res;
    }

    // Logically AND n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 1 0
    fn alu_and_n(&mut self, n: u8) {
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
    fn alu_or_n(&mut self, n: u8) {
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
    fn alu_xor_n(&mut self, n: u8) {
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
    fn alu_cp_n(&mut self, n: u8) {
        let a = self.reg.a;
        self.alu_sub_n(n);
        self.reg.a = a;
    }

    // Increment register n.
    // n = A,B,C,D,E,H,(HL)
    // Flags
    // Z 0 H -
    fn alu_inc_n(&mut self, n: u8) -> u8 {
        let res = n.wrapping_add(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, n & 0xf == 0xf);
        res
    }

    // Decrement register n.
    // n = A,B,C,D,E,H,(HL)
    fn alu_dec_n(&mut self, n: u8) -> u8 {
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
    fn alu_add_hl_nn(&mut self, nn: u16) {
        let res = u32::from(self.reg.hl()) + u32::from(nn);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, unimplemented!());
        self.reg.set_flag(C, unimplemented!());
        self.reg.set_hl((res & 0xffff) as u16);
    }

    // Add n to Stack Pointer (SP).
    // n = signed #
    // Flags
    // 0 0 H C
    fn alu_add_sp_n(&mut self, n: i8) {
        unimplemented!()
    }

    // Increment register nn.
    // n = BC,DE,HL,SP
    fn alu_inc_nn(&mut self, nn: u16) -> u16 {
        nn.wrapping_add(1)
    }

    // Decrement register nn.
    // n = BC,DE,HL,SP
    fn alu_dec_nn(&mut self, nn: u16) -> u16 {
        nn.wrapping_sub(1)
    }

    // Rotate n left. Old bit 7 to Carry flag
    // Flags:
    // Z 0 0 C
    fn alu_rlc_n(&mut self, n: u8) -> u8 {
        let res = n << 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x80 != 0);
        res
    }

    // Rotate n left through Carry flag.
    // Flags:
    // Z 0 0 C
    fn alu_rl_n(&mut self, n: u8) -> u8 {
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
    fn alu_rrc_n(&mut self, n: u8) -> u8 {
        let res = n >> 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, n & 0x1 != 0);
        res
    }

    // Rotate n right through Carry flag.
    // Flags:
    // Z 0 0 C
    fn alu_rr_n(&mut self, n: u8) -> u8 {
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
    fn alu_bit_b_n(&mut self, b: u8, n: u8) {
        assert!(b >= 0 && b <= 7);
        self.reg.set_flag(Z, n & (1 << b) != 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }

    // Pushes present address onto stack.
    // Jump to address $000 + n
    // n = 00,$08,$10,$18,$20,$28,$30,$38
    fn rst_n(&mut self, n: u8, mmu: &mut Mmu) {
        self.stack_push(self.reg.pc, mmu);
        self.reg.pc = n as u16;
    }

    // Call Address n if following condition is true:
    // c = NZ, Call if Z flag is reset.
    // c = Z, Call if Z flag is set.
    // c = NC, Call if C flag is reset.
    // c = C, Call if C flag is set.
    fn call_c_n(&mut self, c: bool, mmu: &mut Mmu) {
        let n = self.fetch_word(mmu);
        if c {
            self.stack_push(self.reg.pc, mmu);
            self.reg.pc = n;
        }
    }

    // Push address of next instruction onto the stack and then jump to address n.
    fn call_n(&mut self, mmu: &mut Mmu) {
        let n = self.fetch_word(mmu);
        self.stack_push(self.reg.pc, mmu);
        self.reg.pc = n;
    }

    // Jump to address n if following condition is true:
    // c = NZ, Call if Z flag is reset.
    // c = Z, Call if Z flag is set.
    // c = NC, Call if C flag is reset.
    // c = C, Call if C flag is set.
    fn jp_c_n(&mut self, c: bool, mmu: &mut Mmu) {
        let n = self.fetch_word(mmu);
        if c {
            self.reg.pc = n;
        }
    }

    // Add n to current address and jump tp it.
    fn jr_c(&mut self, c: bool, mmu: &mut Mmu) {
        let n = self.fetch_signed(mmu);
        if c {
            let pc = i32::from(self.reg.pc) + i32::from(n);
            self.reg.pc = (pc & 0xffff) as u16;
        }
    }
}

impl Cpu {
    pub fn step(&mut self, mmu: &mut Mmu) -> usize {
        self.exec(mmu)
    }

    fn exec(&mut self, mmu: &mut Mmu) -> usize {
        let opcode = self.fetch(mmu);
        match opcode {
            // ADD A,n
            0x80 => self.alu_add_n(self.reg.b),
            0x81 => self.alu_add_n(self.reg.c),
            0x82 => self.alu_add_n(self.reg.d),
            0x83 => self.alu_add_n(self.reg.e),
            0x84 => self.alu_add_n(self.reg.h),
            0x85 => self.alu_add_n(self.reg.l),
            0x86 => self.alu_add_n(mmu.read(self.reg.hl())),
            0x87 => self.alu_add_n(self.reg.a),
            0xc6 => {
                let d8 = self.fetch(mmu);
                self.alu_add_n(d8)
            }
            // ADC A,n
            0x88 => self.alu_adc_n(self.reg.b),
            0x89 => self.alu_adc_n(self.reg.c),
            0x8a => self.alu_adc_n(self.reg.d),
            0x8b => self.alu_adc_n(self.reg.e),
            0x8c => self.alu_adc_n(self.reg.h),
            0x8d => self.alu_adc_n(self.reg.l),
            0x8e => self.alu_adc_n(mmu.read(self.reg.hl())),
            0x8f => self.alu_adc_n(self.reg.a),
            0xce => {
                let d8 = self.fetch(mmu);
                self.alu_adc_n(d8)
            }
            // SUB n
            0x90 => self.alu_sub_n(self.reg.b),
            0x91 => self.alu_sub_n(self.reg.c),
            0x92 => self.alu_sub_n(self.reg.d),
            0x93 => self.alu_sub_n(self.reg.e),
            0x94 => self.alu_sub_n(self.reg.h),
            0x95 => self.alu_sub_n(self.reg.l),
            0x96 => self.alu_sub_n(mmu.read(self.reg.hl())),
            0x97 => self.alu_sub_n(self.reg.a),
            0xd6 => {
                let d8 = self.fetch(mmu);
                self.alu_sub_n(d8)
            }
            // ADC A,n
            0x98 => self.alu_sbc_n(self.reg.b),
            0x99 => self.alu_sbc_n(self.reg.c),
            0x9a => self.alu_sbc_n(self.reg.d),
            0x9b => self.alu_sbc_n(self.reg.e),
            0x9c => self.alu_sbc_n(self.reg.h),
            0x9d => self.alu_sbc_n(self.reg.l),
            0x9e => self.alu_sbc_n(mmu.read(self.reg.hl())),
            0x9f => self.alu_sbc_n(self.reg.a),
            0xde => {
                let d8 = self.fetch(mmu);
                self.alu_sbc_n(d8)
            }
            // AND n
            0xa0 => self.alu_and_n(self.reg.b),
            0xa1 => self.alu_and_n(self.reg.c),
            0xa2 => self.alu_and_n(self.reg.d),
            0xa3 => self.alu_and_n(self.reg.e),
            0xa4 => self.alu_and_n(self.reg.h),
            0xa5 => self.alu_and_n(self.reg.l),
            0xa6 => self.alu_and_n(mmu.read(self.reg.hl())),
            0xa7 => self.alu_and_n(self.reg.a),
            0xe6 => {
                let d8 = self.fetch(mmu);
                self.alu_and_n(d8)
            }
            // XOR n
            0xa8 => self.alu_xor_n(self.reg.b),
            0xa9 => self.alu_xor_n(self.reg.c),
            0xaa => self.alu_xor_n(self.reg.d),
            0xab => self.alu_xor_n(self.reg.e),
            0xac => self.alu_xor_n(self.reg.h),
            0xad => self.alu_xor_n(self.reg.l),
            0xae => self.alu_xor_n(mmu.read(self.reg.hl())),
            0xaf => self.alu_xor_n(self.reg.a),
            0xee => {
                let d8 = self.fetch(mmu);
                self.alu_xor_n(d8)
            }
            // OR n
            0xb0 => self.alu_or_n(self.reg.b),
            0xb1 => self.alu_or_n(self.reg.c),
            0xb2 => self.alu_or_n(self.reg.d),
            0xb3 => self.alu_or_n(self.reg.e),
            0xb4 => self.alu_or_n(self.reg.h),
            0xb5 => self.alu_or_n(self.reg.l),
            0xb6 => self.alu_or_n(mmu.read(self.reg.hl())),
            0xb7 => self.alu_or_n(self.reg.a),
            0xf6 => {
                let d8 = self.fetch(mmu);
                self.alu_or_n(d8)
            }
            // CP n
            0xb8 => self.alu_cp_n(self.reg.b),
            0xb9 => self.alu_cp_n(self.reg.c),
            0xba => self.alu_cp_n(self.reg.d),
            0xbb => self.alu_cp_n(self.reg.e),
            0xbc => self.alu_cp_n(self.reg.h),
            0xbd => self.alu_cp_n(self.reg.l),
            0xbe => self.alu_cp_n(mmu.read(self.reg.hl())),
            0xbf => self.alu_cp_n(self.reg.a),
            0xfe => {
                let d8 = self.fetch(mmu);
                self.alu_cp_n(d8)
            }
            // INC n
            0x04 => self.reg.b = self.alu_inc_n(self.reg.b),
            0x14 => self.reg.d = self.alu_inc_n(self.reg.d),
            0x24 => self.reg.h = self.alu_inc_n(self.reg.h),
            0x34 => {
                let hl = self.reg.hl();
                mmu.write(hl, self.alu_inc_n(mmu.read(hl)));
            }
            0x0c => self.reg.c = self.alu_inc_n(self.reg.c),
            0x1c => self.reg.e = self.alu_inc_n(self.reg.e),
            0x2c => self.reg.l = self.alu_inc_n(self.reg.l),
            0x3c => self.reg.a = self.alu_inc_n(self.reg.a),

            // DEC n
            0x05 => self.reg.b = self.alu_dec_n(self.reg.b),
            0x15 => self.reg.d = self.alu_dec_n(self.reg.d),
            0x25 => self.reg.h = self.alu_dec_n(self.reg.h),
            0x35 => {
                let hl = self.reg.hl();
                mmu.write(hl, self.alu_dec_n(mmu.read(hl)));
            }
            0x0d => self.reg.c = self.alu_dec_n(self.reg.c),
            0x1d => self.reg.e = self.alu_dec_n(self.reg.e),
            0x2d => self.reg.l = self.alu_dec_n(self.reg.l),
            0x3d => self.reg.a = self.alu_dec_n(self.reg.a),

            // INC nn
            0x03 => {
                let r = self.alu_inc_nn(self.reg.bc());
                self.reg.set_bc(r)
            }
            0x13 => {
                let r = self.alu_inc_nn(self.reg.de());
                self.reg.set_de(r)
            }
            0x23 => {
                let r = self.alu_inc_nn(self.reg.hl());
                self.reg.set_hl(r)
            }
            0x33 => self.reg.sp = self.alu_inc_nn(self.reg.sp),
            // DEC nn
            0x0b => {
                let r = self.alu_dec_nn(self.reg.bc());
                self.reg.set_bc(r)
            }
            0x1b => {
                let r = self.alu_dec_nn(self.reg.de());
                self.reg.set_de(r)
            }
            0x2b => {
                let r = self.alu_dec_nn(self.reg.hl());
                self.reg.set_hl(r)
            }
            0x3b => self.reg.sp = self.alu_dec_nn(self.reg.sp),
            // ADD HL,nn
            0x09 => self.alu_add_hl_nn(self.reg.bc()),
            0x19 => self.alu_add_hl_nn(self.reg.de()),
            0x29 => self.alu_add_hl_nn(self.reg.hl()),
            0x39 => self.alu_add_hl_nn(self.reg.sp),

            0xe8 => unimplemented!("ADD SP,r8"),

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
                self.reg.set_flag(Z, unimplemented!());
                self.reg.set_flag(H, false);
                self.reg.set_flag(C, unimplemented!());

                unimplemented!("Implement 0x27 DAA")
            }
            // CPL
            0x2f => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
            }

            // FIXME CONFLICT
            // according to https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
            // Z is reset, but according to http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
            // Z depends on the result
            0x07 => self.reg.a = self.alu_rlc_n(self.reg.a), // unimplemented!("RLCA"),
            0x17 => self.reg.a = self.alu_rl_n(self.reg.a),  // unimplemented!("RLA"),
            0x0f => self.reg.a = self.alu_rrc_n(self.reg.a), // unimplemented!("RRCA"),
            0x1f => self.reg.a = self.alu_rr_n(self.reg.a),  // unimplemented!("RRA"),

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
                unimplemented!("verify that this is correct (endianness)");
                // let a16 = self.fetch_word(mmu);
                // mmu.write_word(a16, self.reg.sp);
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
            0xf8 => unimplemented!("LD HL,SP+r8"),
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
            0xfa => unimplemented!("LD A,(a16)"),

            // POP nn
            0xc1 | 0xd1 | 0xe1 => {
                let r = self.stack_pop(mmu);
                match opcode {
                    0xc1 => self.reg.set_bc(r),
                    0xd1 => self.reg.set_de(r),
                    0xe1 => self.reg.set_hl(r),
                    _ => panic!(),
                }
            }
            0xf1 => unimplemented!("POP AF => F set to 0x0 afterwards?"),
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
                self.reg.pc = self.stack_pop(mmu);
                self.ime = true
            }

            0xc7 => self.rst_n(0x00, mmu),
            0xd7 => self.rst_n(0x10, mmu),
            0xe7 => self.rst_n(0x20, mmu),
            0xf7 => self.rst_n(0x30, mmu),
            0xcf => self.rst_n(0x08, mmu),
            0xdf => self.rst_n(0x18, mmu),
            0xef => self.rst_n(0x28, mmu),
            0xff => self.rst_n(0x38, mmu),

            0xc4 => self.call_c_n(!self.reg.is_flag(Z), mmu),
            0xd4 => self.call_c_n(!self.reg.is_flag(C), mmu),
            0xcc => self.call_c_n(self.reg.is_flag(Z), mmu),
            0xdc => self.call_c_n(self.reg.is_flag(C), mmu),
            0xcd => self.call_n(mmu),

            0xc2 => self.jp_c_n(!self.reg.is_flag(Z), mmu),
            0xd2 => self.jp_c_n(!self.reg.is_flag(C), mmu),
            0xca => self.jp_c_n(self.reg.is_flag(Z), mmu),
            0xda => self.jp_c_n(self.reg.is_flag(C), mmu),
            0xc3 => self.reg.pc = self.fetch_word(mmu),
            0xe9 => {
                panic!("PDF says: Jump to address contained in HL. Is it the register, or the pointed memory?");
                self.reg.pc = mmu.read_word(self.reg.hl())
            }

            0x20 => self.jr_c(!self.reg.is_flag(Z), mmu), // unimplemented!("JR NZ,r8"),
            0x30 => self.jr_c(!self.reg.is_flag(C), mmu), // unimplemented!("JR NC,r8"),
            0x28 => self.jr_c(self.reg.is_flag(Z), mmu),  // unimplemented!("JR Z,r8"),
            0x38 => self.jr_c(self.reg.is_flag(C), mmu),  // unimplemented!("JR C,r8"),
            0x18 => self.jr_c(true, mmu),                 // unimplemented!("JR r8"),

            // Misc/control instructions
            0x00 => {}                                                 // NOP
            0x10 => unimplemented!("0x10 - STOP 0 - not implemented"), // STOP 0
            0x76 => self.halt = true,
            0xf3 => self.ime = false,
            0xfb => self.ime = true,
            0xcb => {
                let cb = self.fetch(mmu);
                match cb {
                    0x7c => self.alu_bit_b_n(7, self.reg.h),
                    0x11 => self.reg.c = self.alu_rl_n(self.reg.c),
                    _ => unimplemented!("{:x}", self.reg.pc - 2),
                }
                //unimplemented!("0xcb")
            }

            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb..=0xed | 0xf4 | 0xfc | 0xfd => {
                panic!("Undefined opcode = 0x{:02x}", opcode)
            }
        }

        0
    }
}

#[cfg(test)]
mod test {}
