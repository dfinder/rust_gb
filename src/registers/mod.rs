pub mod registers {

    //use crate::memory::memory::{self, MemoryStruct};
    use crate::memory_wrapper::memory_wrapper::MemWrap;
    /**pub fn sign_8(x: u8)-> i8{
        ((!x)+1) as i8
    }**/
    #[derive(Copy, Clone)]
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
    #[derive(Copy, Clone)]
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
        A: u8, //A IS THE ACCUMULATOR
        B: u8,
        C: u8,
        D: u8,
        E: u8,
        F: u8,
        H: u8,
        L: u8,
        SP: u16,
        pc: u16,
        //memory: &'static mut memory::MemoryStruct
    }
    pub enum FlagR {
        Set,
        Unset,
        Keep,
        Function(bool), //What should the argument here be?
    }
    impl RegStruct {
        pub fn get_acc(&mut self) -> u8 {
            self.A
        }
        pub fn set_acc(&mut self, val: u8) {
            self.A = val
        }
        pub fn get_r8(&mut self, reg: SingleReg, memory: &mut MemWrap) -> u8 {
            match reg {
                SingleReg::A => self.A,
                SingleReg::B => self.B,
                SingleReg::C => self.C,
                SingleReg::D => self.D,
                SingleReg::E => self.E,
                SingleReg::F => self.F,
                SingleReg::H => self.H,
                SingleReg::Memptr => memory.grab_memory_8(self.get_r16(DoubleReg::HL)),
                SingleReg::L => self.L,
            }
        }
        pub fn get_r16(&mut self, reg: DoubleReg) -> u16 {
            let glue = |x: u8, y: u8| x as u16 * 0x80 + y as u16;
            match reg {
                DoubleReg::AF => glue(self.A, self.F),
                DoubleReg::BC => glue(self.B, self.C),
                DoubleReg::DE => glue(self.D, self.E),
                DoubleReg::HL | DoubleReg::HLP | DoubleReg::HLM => glue(self.H, self.L),
                DoubleReg::SP => self.SP,
                DoubleReg::PC => self.pc,
            }
        }

        pub fn set_r8(&mut self, reg: SingleReg, val: u8, memory: &mut MemWrap) {
            match reg {
                SingleReg::A => self.A = val,
                SingleReg::B => self.B = val,
                SingleReg::C => self.C = val,
                SingleReg::D => self.D = val,
                SingleReg::E => self.E = val,
                SingleReg::Memptr => memory.set_memory_8(self.get_r16(DoubleReg::HL), val),
                SingleReg::F => panic!("oopsie, we tried to set F"),
                SingleReg::H => self.H = val,
                SingleReg::L => self.L = val,
            }
        }
        pub fn set_r16(&mut self, reg: DoubleReg, val: u16, memory: &mut MemWrap) {
            let glue = |x: u8, y: u8| x as u16 * 0x80 + y as u16;
            match reg {
                DoubleReg::AF => panic!("We can't set F from Double Set"),
                DoubleReg::BC => {
                    self.B = (val >> 8) as u8;
                    self.C = val as u8;
                }
                DoubleReg::DE => {
                    self.D = (val >> 8) as u8;
                    self.E = val as u8;
                }
                DoubleReg::HL => {
                    self.H = (val >> 8) as u8;
                    self.L = val as u8;
                }
                DoubleReg::HLP => {
                    let mut addr = glue(self.H, self.L);
                    memory.set_memory_16(addr, val);
                    addr += 1;
                    self.H = (addr >> 8) as u8;
                    self.L = addr as u8;
                }
                DoubleReg::HLM => {
                    let mut addr = glue(self.H, self.L);
                    memory.set_memory_16(addr, val);
                    addr -= 1;
                    self.H = (addr >> 8) as u8;
                    self.L = addr as u8;
                }
                DoubleReg::SP => self.SP = val,
                DoubleReg::PC => self.pc = val,
            }
        }
        pub fn set_flag(&mut self, flag: Flag) {
            self.F |= match flag {
                Flag::Zero => 128,
                Flag::Neg => 64,
                Flag::HalfCarry => 32,
                Flag::Carry => 16,
            }
        }

        pub fn flag_cond(&mut self, flag: Flag, b: bool) {
            if b {
                self.set_flag(flag);
            } else {
                self.unset_flag(flag);
            }
        }
        pub fn set_flags(&mut self, zero: bool, neg: bool, hc: bool, carry: bool) {
            self.F = (((((zero as u8) + ((neg as u8) << 1)) + hc as u8) << 1) + carry as u8) << 4
        }
        pub fn unset_flag(&mut self, flag: Flag) {
            self.F &= match flag {
                Flag::Zero => 0x7F,
                Flag::Neg => 0xBF,
                Flag::HalfCarry => 0xDF,
                Flag::Carry => 0xEF,
            }
        }
        pub fn inc_pc(&mut self, amount: u8) -> u16 {
            self.pc += amount as u16;
            self.pc
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
        pub fn change_r16(&mut self, reg: DoubleReg, fun: &dyn Fn(u16) -> u16, mem: &mut MemWrap) {
            let result = fun(self.get_r16(reg));
            self.set_r16(reg, result, mem);
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
            //Sets the initial states of registers???
            return Self {
                A: 0, //A IS THE ACCUMULATOR
                B: 0,
                C: 0,
                D: 0,
                E: 0,
                F: 0,
                H: 0,
                L: 0,
                SP: 0,
                pc: 0,
            };
        }
    }
}
