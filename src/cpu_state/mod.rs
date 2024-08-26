
use crate::registers::registers::*;
pub mod cpu_state{

    use crate::memory_wrapper::{memory_wrapper::MemWrap, vram::vram::Vram};

    use super::{DoubleReg, Flag, RegStruct, SingleReg};

    pub struct CpuState{
        memory: MemWrap,
        registers: RegStruct
    }
    impl CpuState{
        pub fn new()->Self{
            Self{
                memory:MemWrap::new(),
                registers:RegStruct::new(),
            }
        }
        //pub fn get_bit(&mut self, )
        pub fn get_r8_mid(&mut self,opcode:u8)->SingleReg{
            self.get_r8_end(opcode>>3)
        }
        pub fn get_r8_end(&mut self,opcode:u8)->SingleReg{
            match opcode % 8 {
                0=>SingleReg::B,
                1=>SingleReg::C,
                2=>SingleReg::D,
                3=>SingleReg::E,
                4=>SingleReg::H, 
                5=>SingleReg::L,
                6=>SingleReg::Memptr,
                7=>SingleReg::A,
                _=>unreachable!()
            }
        }
        pub fn r16_tbl(&mut self,opcode: u8)->DoubleReg{
            match (opcode >> 4) % 4{
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::SP,
                _=>unreachable!()
            }
        }
        pub fn r16_stk_tbl(&mut self,opcode:u8)->DoubleReg{
            match (opcode >> 4) % 4{
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::AF,
                _=>unreachable!()
            }
        }
        pub fn r16_mem_tbl(&mut self,opcode:u8)->DoubleReg{
            match (opcode >> 4) % 4{
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HLP,
                3 => DoubleReg::HLM,
                _=>unreachable!()
            }
        }
        pub fn inc_pc(&mut self)->u16{
            self.registers.inc_pc(1)
        }
        pub fn get_acc(&mut self)->u8{
            self.registers.get_r8(SingleReg::A, &mut self.memory)
        }
        pub fn set_acc(&mut self, val:u8){
            self.registers.set_r8(SingleReg::A,val, &mut self.memory)
        }
        pub fn set_pc(&mut self,val: u16){
            self.registers.set_r16(DoubleReg::PC, val, &mut self.memory);
        }
        pub fn change_r8(&mut self, reg:SingleReg, fun: &dyn Fn(u8)-> u8)->u8{
            let val:u8 = fun(self.registers.get_r8(reg,&mut self.memory));
            self.registers.set_r8(reg,val, &mut self.memory);
            val
        }
        pub fn change_r16(&mut self, reg:DoubleReg, fun: &dyn Fn(u16)->u16)->u16{
            let val:u16 = fun(self.registers.get_r16(reg));
            self.registers.set_r16(reg,val, &mut self.memory);
            val
        }        
        /*pub fn get_r16_mem_direct(&mut self,opcode: u8)->u8{
            let reg:DoubleReg = self.r16_mem_tbl(opcode);
            let addr:u16 = self.get_r16_val(reg);
            self.memory.grab_memory_8(addr)
        }*/
        pub fn get_r8_val(&mut self,reg:SingleReg)->u8{
            self.registers.get_r8(reg, &mut self.memory)
        }
        pub fn get_r16_val(&mut self,reg:DoubleReg)->u16{
            self.registers.get_r16(reg)
        }
        pub fn set_r8(&mut self, reg:SingleReg, val:u8){
            self.registers.set_r8(reg, val, &mut self.memory)

        }
        pub fn set_r16_val(&mut self, reg:DoubleReg, val:u16){
            self.registers.set_r16(reg, val, &mut self.memory)
        }
        pub fn set_r16_memory(&mut self,reg:DoubleReg, val:u16){
            self.memory.set_memory_16(self.registers.get_r16(reg), val)
        }
        pub fn get_r16_memory(&mut self,reg:DoubleReg)->u8{
            self.memory.grab_memory_8(self.registers.get_r16(reg))
        }
        pub fn get_r16_memory_word(&mut self, reg:DoubleReg)->u16{
            self.memory.grab_memory_16(self.registers.get_r16(reg))
        }
        /*pub fn get_reg_val(&mut self,opcode:u8)->u8 {
            let reg = self.get_r8_end(opcode);
            self.get_r8_val(reg)
        }*/
        pub fn get_flag(&mut self, flag: Flag) -> bool {
            let flag_reg = self.registers.get_r8(SingleReg::F, &mut self.memory);
            match flag{
                Flag::Zero => (flag_reg & 0x80) == 0x80,
                Flag::Neg => (flag_reg & 0x40) == 0x40,
                Flag::HalfCarry => (flag_reg & 0x20) == 0x20,
                Flag::Carry => (flag_reg & 0x10) == 0x10,
            }
        }
        pub fn set_flag(&mut self,flag:Flag,state:bool){
            if state{
                self.registers.set_flag(flag);
            }
            else{
                self.registers.unset_flag(flag);
            }

        }
        pub fn set_flags(&mut self, zero:bool, neg:bool, hc:bool, carry:bool){
            self.set_flag(Flag::Zero, zero);
            self.set_flag(Flag::Neg, neg);
            self.set_flag(Flag::HalfCarry, hc);
            self.set_flag(Flag::Carry, carry);
        }
        pub fn apply_fun_to_acc(&mut self, fun: &dyn Fn(u8)->u8)->u8{
            self.registers.change_r8(SingleReg::A,fun, &mut self.memory)
        }
        pub fn set_byte(&mut self,addr:u16,val:u8){
            self.memory.set_memory_8(addr, val)
        }
        pub fn get_byte(&mut self,addr:u16)->u8{
            self.memory.grab_memory_8(addr)
        }
        pub fn get_imm8(&mut self)->u8{
            self.memory.grab_memory_8(self.registers.inc_pc(1))
        }
        pub fn get_imm16(&mut self)->u16{
            let ret:u16 = self.memory.grab_memory_16(self.registers.inc_pc(1));
            self.registers.inc_pc(1);
            return ret;
        }
        pub fn get_cond(&mut self,opcode:u8)->bool{
            match (opcode >> 4) % 4{
                0 => !self.get_flag(Flag::Zero),
                1 => self.get_flag(Flag::Zero),
                2 => !self.get_flag(Flag::Carry),
                3 => self.get_flag(Flag::Carry),
                _=>unreachable!()
            }
        }
        pub fn flip_carry(&mut self){
            self.set_flag(Flag::Neg,false);
            self.set_flag(Flag::HalfCarry, false);
            if self.get_flag(Flag::Carry){
                self.set_flag(Flag::Carry,false);
            }else{
                self.set_flag(Flag::Carry,true);
            }
        }
        pub fn get_half_word(&mut self,addr:u16)->u16{
            self.memory.grab_memory_16(addr)
        }
        pub fn set_half_word(&mut self,addr:u16, value:u16){
            self.memory.set_memory_16(addr, value)
        }
        pub fn get_graphics(&mut self) -> &Vram{
            self.memory.get_vram()
        }
    }
}