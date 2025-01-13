pub mod registers {

    use log::info;

    //use crate::memory::memory::{self, MemoryStruct};
    use std::fmt::Debug;

    use crate::memory::memory_wrapper::MemWrap;

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
        Neg, //Often marksed as N.
        HalfCarry,
        Carry,
    }
    pub struct RegStruct {
        a: u8, //A IS THE ACCUMULATOR
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        sp: u16,
        pc: u16,
        //memory: &'static mut memory::MemoryStruct
    }
    impl Debug for RegStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Reg")
                .field("A", &format!("{:X?}", &self.a))
                .field("B", &format!("{:X?}", &self.b))
                .field("C", &format!("{:X?}", &self.c))
                .field("D", &format!("{:X?}", &self.d))
                .field("E", &format!("{:X?}", &self.e))
                .field(
                    "F",
                    &format!(
                        "[Z:{},N:{},HC:{},C:{}]",
                        ((&self.f >> 7) == 1) as u8,
                        (((&self.f & 0x40) >> 6) == 1) as u8,
                        (((&self.f & 0x20) >> 5) == 1) as u8,
                        (((&self.f & 0x10) >> 4) == 1) as u8
                    ),
                )
                .field("H", &format!("{:X?}", &self.h))
                .field("L", &format!("{:X?}", &self.l))
                .field("SP", &format!("{:X?}", &self.sp))
                .field("pc", &format!("{:X?}", &self.pc))
                .finish()
        }
    }
    impl RegStruct {
        pub fn get_acc(&mut self) -> u8 {
            self.a
        }
        pub fn set_acc(&mut self, val: u8) {
            self.a = val
        }
        pub fn get_r8(&mut self, reg: SingleReg, memory: &mut MemWrap) -> u8 {
            match reg {
                SingleReg::A => self.a,
                SingleReg::B => self.b,
                SingleReg::C => self.c,
                SingleReg::D => self.d,
                SingleReg::E => self.e,
                SingleReg::F => self.f,
                SingleReg::H => self.h,
                SingleReg::Memptr => memory.grab_memory_8(self.get_r16(DoubleReg::HL)),
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
                    //info!("ADDR {:X?}", addr);
                    self.h = (addr >> 8) as u8;
                    self.l = addr as u8;
                    addr + 1
                }
                DoubleReg::SP => self.sp,
                DoubleReg::PC => self.pc,
            }
        }

        pub fn set_r8(&mut self, reg: SingleReg, val: u8, memory: &mut MemWrap) {
            match reg {
                SingleReg::A => self.a = val,
                SingleReg::B => self.b = val,
                SingleReg::C => self.c = val,
                SingleReg::D => self.d = val,
                SingleReg::E => self.e = val,
                SingleReg::Memptr => memory.set_memory_8(self.get_r16(DoubleReg::HL), val),
                SingleReg::F => panic!("oopsie, we tried to set F"),
                SingleReg::H => self.h = val,
                SingleReg::L => self.l = val,
            }
        }
        pub fn set_r16(&mut self, reg: DoubleReg, val: u16) {
            match reg {
                DoubleReg::AF => panic!("We can't set F from Double Set"),
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
                DoubleReg::HLP => panic!("This doesn't work like how we want it to"),
                /* {

                    let mut addr = glue(self.h, self.l);
                    memory.set_memory_16(addr, val);
                    addr += 1;
                    self.h = (addr >> 8) as u8;
                    self.l = addr as u8;
                } */
                DoubleReg::HLM => panic!("This doesn't work like how we want it to"), /*
                let mut addr = glue(self.h, self.l);
                memory.set_memory_16(addr, val);
                addr -= 1;
                self.h = (addr >> 8) as u8;
                self.l = addr as u 8;
                }*/
                DoubleReg::SP => self.sp = val,
                DoubleReg::PC => self.pc = val,
            }
        }
        pub fn set_flag(&mut self, flag: Flag) {

            //dbg!(self.f);
            self.f |= match flag {
                Flag::Zero => 128,
                Flag::Neg => 64,
                Flag::HalfCarry => 32,
                Flag::Carry => 16,
            };
            //dbg!(self.f);
        }

        pub fn flag_cond(&mut self, flag: Flag, b: bool) {
            if b {
                self.set_flag(flag);
            } else {
                self.unset_flag(flag);
            }
        }
        pub fn set_flags(&mut self, zero: bool, neg: bool, hc: bool, carry: bool) {
            self.f =
                ((((((zero as u8) << 1) + ((neg as u8) << 1)) + hc as u8) << 1) + carry as u8) << 4
        }
        pub fn unset_flag(&mut self, flag: Flag) {
            self.f &= match flag {
                Flag::Zero => 0x7F,
                Flag::Neg => 0xBF,
                Flag::HalfCarry => 0xDF,
                Flag::Carry => 0xEF,
            }
        }
        pub fn inc_pc(&mut self, amount: u8) -> u16 {
            self.pc += amount as u16;
            return self.pc;
        }
        pub fn set_pc(&mut self, amount: u16) -> u16 {
            self.pc = amount;
            self.pc
        }
        pub fn change_r8(
            &mut self,
            reg: SingleReg,
            fun: &dyn Fn(u8) -> u8,
            mem: &mut MemWrap,
        ) -> u8 {
            let result = fun(self.get_r8(reg, mem));
            self.set_r8(reg, result, mem);
            result
        }
        pub fn change_r16(&mut self, reg: DoubleReg, fun: &dyn Fn(u16) -> u16) {
            let result = fun(self.get_r16(reg));
            self.set_r16(reg, result);
        }
        /**pub fn apply_fun_to_reg(&mut self, reg:SingleReg,fun: &dyn Fn(u8)->u8){
            let acc:u8 = self.get_r8(SingleReg::A);
            let reg_val:u8 = self.get_r8(reg);
            self.set_r8(SingleReg::A,fun(acc,reg_val))
        }**/
        pub fn get_bit(&mut self, reg: SingleReg, idx: u8, mem: &mut MemWrap) -> bool {
            return (self.get_r8(reg, mem) & (1 << idx)) == (1 << idx);
        }
        pub fn reset_flags(&mut self) {
            self.set_flags(false, false, false, false);
        }
        pub fn new() -> Self {
            return Self {
                a: 0x00, //A IS THE ACCUMULATOR
                b: 0,
                c: 0x00,
                d: 0,
                e: 0x00,
                f: 0x00,
                h: 0x00,
                l: 0x00,
                sp: 0x0000,
                pc: 0x0000,
            };
        }
    }
}
