pub mod cpu_state {
    use crate::{
        audio::audio_controller::AudioController, joypad::joypad::Joypad,
        memory::memory_wrapper::MemWrap,
    };
    
    use sdl2::{render::Canvas, video::Window};
    use std::fmt::Debug;
    use std::{cell::RefCell, fs::File, rc::Rc};

    pub struct CpuState {
        pub memory: MemWrap,
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        sp: u16,
        pc: u16,
    }
    impl CpuState {
        pub fn new(
            joypad: Joypad,
            audio_con: AudioController,
            wait_ref: Rc<RefCell<u8>>,
            canvas: Canvas<Window>,
            cartridge: File,
        ) -> Self {
            Self {
                memory: MemWrap::new(joypad, audio_con, canvas, wait_ref, cartridge),
                a: 0x00,
                b: 0x00,
                c: 0x00,
                d: 0x00,
                e: 0x00,
                f: 0x00,
                h: 0x00,
                l: 0x00,
                sp: 0x0000,
                pc: 0x0000,
            }
        }
        pub fn get_r8_end(&mut self, opcode: u8) -> SingleReg {
            match opcode % 8 {
                0 => SingleReg::B,
                1 => SingleReg::C,
                2 => SingleReg::D,
                3 => SingleReg::E,
                4 => SingleReg::H,
                5 => SingleReg::L,
                6 => SingleReg::Memptr,
                7 => SingleReg::A,
                _ => unreachable!(),
            }
        }
        pub fn r16_tbl(&mut self, opcode: u8) -> DoubleReg {
            let ret = match (opcode >> 4) % 4 {
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::SP,
                _ => unreachable!(),
            };
            ret
        }
        pub fn r16_stk_tbl(&mut self, opcode: u8) -> DoubleReg {
            match (opcode >> 4) % 4 {
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::AF,
                _ => unreachable!(),
            }
        }
        pub fn r16_mem_tbl(&mut self, opcode: u8) -> DoubleReg {
            match (opcode >> 4) % 4 {
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HLP,
                3 => DoubleReg::HLM,
                _ => unreachable!(),
            }
        }
        pub fn inc_pc(&mut self) -> u16 {
            self.pc += 1 as u16;
            self.pc
        }
        pub fn get_pc(&mut self) -> u16 {
            self.pc
        }
        pub fn set_pc(&mut self, amount: u16) {
            self.pc = amount;
        }
        pub fn set_r16_mem_8(&mut self, reg: DoubleReg, val: u8) {
            let reg_val = self.get_r16(reg);
            self.memory.set_memory_8(reg_val, val)
        }
        pub fn set_r16_mem_16(&mut self, reg: DoubleReg, val: u16) {
            let reg_val = self.get_r16(reg);
            self.memory.set_memory_16(reg_val, val)
        }
        pub fn get_r16_mem_8(&mut self, reg: DoubleReg) -> u8 {
            let addr = self.get_r16(reg);
            //info!("{:X?}",addr);
            let ret = self.memory.grab_memory_8(addr);
            //info!("{:X?}",ret );
            ret 
        }
        pub fn get_r16_mem_16(&mut self, reg: DoubleReg) -> u16 {
            let addr = self.get_r16(reg);
            self.memory.grab_memory_16(addr)
        }
        pub fn pop(&mut self)->u16{
            
            let value = self.get_r16_mem_16(DoubleReg::SP);
            self.sp=self.sp+2;
            //info!("POPPING {:?}",value);
            return value
        }
        pub fn push(&mut self,val: u16){

            //info!("PUSHING {:?}",val);
            self.sp=self.sp-2;
            self.set_r16_mem_16(DoubleReg::SP, val);
        }
        pub fn get_flag(&mut self, flag: Flag) -> bool {
            match flag {
                Flag::Zero => (self.f & 0x80) == 0x80,
                Flag::Neg => (self.f & 0x40) == 0x40,
                Flag::HalfCarry => (self.f & 0x20) == 0x20,
                Flag::Carry => (self.f & 0x10) == 0x10,
            }
        }
        pub fn mark_flag(&mut self, flag: Flag, state: bool) {
            if state {
                self.set_flag(flag);
            } else {
                self.unset_flag(flag);
            }
        }
        pub fn set_byte(&mut self, addr: u16, val: u8) {
            self.memory.set_memory_8(addr, val)
        }
        pub fn get_byte(&mut self, addr: u16) -> u8 {
            self.memory.grab_memory_8(addr)
        }
        pub fn get_imm8(&mut self) -> u8 {
            let pc = self.inc_pc();
            self.memory.grab_memory_8(pc)
        }
        pub fn get_simm8(&mut self) -> i8 {
            let pc = self.inc_pc();
            self.memory.grab_memory_8(pc) as i8
        }
        pub fn get_imm16(&mut self) -> u16 {
            let pc = self.inc_pc();
            let ret: u16 = self.memory.grab_memory_16(pc);
            self.inc_pc();
            return ret;
        }
        pub fn flip_carry(&mut self) {
            self.mark_flag(Flag::Neg, false);
            self.mark_flag(Flag::HalfCarry, false);
            if self.get_flag(Flag::Carry) {
                self.mark_flag(Flag::Carry, false);
            } else {
                self.mark_flag(Flag::Carry, true);
            }
        }
        pub fn get_mem_16(&mut self, addr: u16) -> u16 {
            self.memory.grab_memory_16(addr)
        }
        pub fn set_mem_16(&mut self, addr: u16, value: u16) {
            self.memory.set_memory_16(addr, value)
        }
        pub fn on_clock(&mut self) {
            self.memory.on_clock();
        }
        pub fn get_acc(&mut self) -> u8 {
            self.a
        }
        pub fn set_acc(&mut self, val: u8) {
            self.a = val
        }
        pub fn get_r8(&mut self, reg: SingleReg) -> u8 {
            match reg {
                SingleReg::A => self.a,
                SingleReg::B => self.b,
                SingleReg::C => self.c,
                SingleReg::D => self.d,
                SingleReg::E => self.e,
                SingleReg::F => self.f,
                SingleReg::H => self.h,
                SingleReg::Memptr => {
                    let addr = self.get_r16(DoubleReg::HL);
                    self.memory.grab_memory_8(addr)
                }
                SingleReg::L => self.l,
            }
        }
        pub fn get_r16(&mut self, reg: DoubleReg) -> u16 {
            let glue = |x: u8, y: u8| x as u16 * 0x100 + y as u16;
            match reg {
                DoubleReg::AF => glue(self.a, self.f),
                DoubleReg::BC => glue(self.b, self.c),
                DoubleReg::DE => glue(self.d, self.e),
                DoubleReg::HL => glue(self.h, self.l),
                DoubleReg::HLP => {
                    let mut addr = glue(self.h, self.l);
                    addr += 1;
                    self.h = (addr >> 8) as u8;
                    self.l = addr as u8;
                    addr - 1
                }
                DoubleReg::HLM => {
                    let mut addr = glue(self.h, self.l);
                    addr -= 1;
                    self.h = (addr >> 8) as u8;
                    self.l = addr as u8;
                    addr + 1
                }
                DoubleReg::SP => self.sp,
                DoubleReg::PC => self.pc,
            }
        }
        pub fn set_r8(&mut self, reg: SingleReg, val: u8) {
            match reg {
                SingleReg::A => self.a = val,
                SingleReg::B => self.b = val,
                SingleReg::C => self.c = val,
                SingleReg::D => self.d = val,
                SingleReg::E => self.e = val,
                SingleReg::Memptr => {
                    let addr = self.get_r16(DoubleReg::HL);
                    self.memory.set_memory_8(addr, val)
                }
                SingleReg::F => panic!("We cannot set the flag register"),
                SingleReg::H => self.h = val,
                SingleReg::L => self.l = val,
            }
        }
        pub fn set_r16(&mut self, reg: DoubleReg, val: u16) {
            match reg {
                DoubleReg::AF|DoubleReg::HLP | DoubleReg::HLM=> panic!("We can't set F from Double Set"),
                DoubleReg::BC => {
                    self.b = (val >> 8) as u8;
                    self.c = val as u8;
                }
                DoubleReg::DE => {
                    self.d = (val >> 8) as u8;
                    self.e = val as u8;
                }
                DoubleReg::HL => {
                    self.h = (val >> 8) as u8;
                    self.l = val as u8;
                }
                DoubleReg::SP => self.sp = val,
                DoubleReg::PC => self.pc = val,
            }
        }
        pub fn set_flag(&mut self, flag: Flag) {
            self.f |= match flag {
                Flag::Zero => 128,
                Flag::Neg => 64,
                Flag::HalfCarry => 32,
                Flag::Carry => 16,
            };
        }
        pub fn set_flags(&mut self, zero: bool, neg: bool, hc: bool, carry: bool) {
            
            self.f = (128 * zero as u8) + (64*neg as u8) + (32 * hc as u8) + 16*(carry as u8);
            //println!("{:X?}",self.f);
        }
        pub fn unset_flag(&mut self, flag: Flag) {
            self.f &= match flag {
                Flag::Zero => 0x7F,
                Flag::Neg => 0xBF,
                Flag::HalfCarry => 0xDF,
                Flag::Carry => 0xEF,
            };
        }
        pub fn change_r16(&mut self, reg: DoubleReg, fun: &dyn Fn(u16) -> u16) {
            let result = fun(self.get_r16(reg));
            self.set_r16(reg, result);
        }
        pub fn get_bit(&mut self, reg: SingleReg, idx: u8) -> bool {
            return (self.get_r8(reg) & (1 << idx)) == (1 << idx);
        }
        pub fn reset_flags(&mut self) {
            self.set_flags(false, false, false, false);
        }
    }
    impl Debug for CpuState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("")
                .field("A", &format!("{:0>2X?}", &self.a))
                .field("F", &format!("{:0>2X?}", &self.f))
                .field("B", &format!("{:0>2X?}", &self.b))
                .field("C", &format!("{:0>2X?}", &self.c))
                .field("D", &format!("{:0>2X?}", &self.d))
                .field("E", &format!("{:0>2X?}", &self.e))
/*                 .field(
                    "F",
                    &format!(
                        "[Z:{},N:{},HC:{},C:{}]",
                        ((&self.f >> 7) == 1) as u8,
                        (((&self.f & 0x40) >> 6) == 1) as u8,
                        (((&self.f & 0x20) >> 5) == 1) as u8,
                        (((&self.f & 0x10) >> 4) == 1) as u8
                    ),
                ) */
                .field("H", &format!("{:0>2X?}", &self.h))
                .field("L", &format!("{:0>2X?}", &self.l))
                .field("SP", &format!("{:0>4X?}", &self.sp))
                .field("PC", &format!("{:0>4X?}", &self.pc))
                .finish()
        }
    }
    #[derive(Copy, Clone, Debug)]
    pub enum SingleReg {
        A,
        B,
        C,
        D,
        E,
        F,
        H,
        L,
        Memptr,
    }
    #[derive(Copy, Clone, Debug)]
    pub enum DoubleReg {
        AF,
        BC,
        DE,
        HL,
        HLP,
        HLM,
        SP,
        PC,
    }
    pub enum Flag {
        Zero,
        Neg, //Often marked as N.
        HalfCarry,
        Carry,
    }
}
