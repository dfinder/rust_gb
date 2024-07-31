
pub mod registers { 

    use crate::cpu;
    use crate::memory::memory::{self, MemoryStruct};
    pub fn set_bit(x: u8, idx: u8, b: bool) -> u8 { // should probably make some file that's just helper functions.
        let mask = !(1 << idx);
        let flag = (b as u8) << idx;
        x & mask | flag
    }
    /**pub fn sign_8(x: u8)-> i8{
        ((!x)+1) as i8
    }**/
    pub enum SingleReg{
        A,
        B,
        C,
        D,
        E,
        F,
        H,
        L,
        Memptr
    }
    pub enum DoubleReg{
        AF,
        BC,
        DE,
        HL,
        HLP,
        HLM,
        SP,
        PC 
    }
    pub enum Flag{
        Zero,
        Neg, //Often marksed as N.
        HalfCarry,
        Carry
    } 
    pub struct RegStruct{
        A:u8, //A IS THE ACCUMULATOR
        B:u8,
        C:u8,
        D:u8,
        E:u8, 
        F:u8, 
        H:u8,
        L:u8,
        SP:u16,
        PC:u16,
        memory: &'static mut memory::MemoryStruct
    }
    pub enum FlagR{
        Set,
        Unset,
        Keep, 
        Function(bool) //What should the argument here be?
    }
    impl RegStruct{
        
        pub fn get_acc(&mut self) -> u8{
            self.A
        }
        pub fn set_acc(&mut self, val:u8){
            self.A=val 
        }
        pub fn get_register(&mut self, reg:SingleReg)  -> u8{
            match reg{ 
                SingleReg::A => self.A,
                SingleReg::B => self.B ,
                SingleReg::C => self.C,
                SingleReg::D => self.D,
                SingleReg::E => self.E,
                SingleReg::F => self.F,
                SingleReg::H => self.H,
                SingleReg::Memptr => {
                    self.memory.grab_memory_8(self.get_double_register(DoubleReg::HL))
                }
                SingleReg::L => self.L,
            }
        }
        pub fn get_double_register(&mut self, reg:DoubleReg) -> u16{
            let glue = |x:u8,y:u8| x as u16 * 0x80 + y as u16;
            match reg{ 
                DoubleReg::AF =>  glue(self.A, self.F),
                DoubleReg::BC =>  glue(self.B, self.C),
                DoubleReg::DE =>  glue(self.D, self.E),
                DoubleReg::HL |DoubleReg::HLP  | DoubleReg::HLM =>  glue(self.H, self.L),
                DoubleReg::SP=> self.SP,
                DoubleReg::PC=> self.PC
            }
        }
        pub fn get_flag(&mut self, flag: Flag) -> bool {
            match flag{
                Flag::Zero => (self.F & 0x80) == 0x80,
                Flag::Neg => (self.F & 0x40) == 0x40,
                Flag::HalfCarry => (self.F & 0x20) == 0x20,
                Flag::Carry => (self.F & 0x10) == 0x10,
            }
        }
        pub fn flip_carry(&mut self){
            self.F ^= 16+32+64+128;
        }
        pub fn set_single_register(&mut self, reg:SingleReg, val:u8){
            match reg{ 
                SingleReg::A => self.A = val ,
                SingleReg::B => self.B = val ,
                SingleReg::C => self.C = val ,
                SingleReg::D => self.D = val ,
                SingleReg::E => self.E = val ,
                SingleReg::Memptr => {
                    self.memory.set_memory_8(self.get_double_register(DoubleReg::HL),val)
                },
                SingleReg::F => panic!("oopsie, we tried to set F"),
                SingleReg::H => self.H = val,
                SingleReg::L => self.L = val,
                _ => unreachable!()
            }
        }
        pub fn set_double_register(&mut self, reg:DoubleReg, val:u16){
            let glue = |x:u8,y:u8| x as u16 * 0x80 + y as u16;
            match reg{ 
                DoubleReg::AF => panic!("We can't set F from Double Set"),
                DoubleReg::BC => {
                    self.B = (val >> 8) as u8;
                    self.C = val as u8;
                },
                DoubleReg::DE => {
                    self.D = (val >> 8) as u8;
                    self.E = val as u8;
                },
                DoubleReg::HL => {
                    self.H = (val >> 8) as u8;
                    self.L = val as u8;
                },
                DoubleReg::HLP => {
                    let new_val = glue(self.H,self.L)+1;
                    self.H = (val >> 8) as u8;
                    self.L = val as u8;
                },
                DoubleReg::HLM => {
                    let new_val = glue(self.H,self.L)-1;
                    self.H = (val >> 8) as u8;
                    self.L = val as u8;
                }
                DoubleReg::SP => self.SP=val,
                DoubleReg::PC => self.PC=val,
            }
        }
        pub fn set_flag(&mut self, flag: Flag){
            self.F |= match flag{
                Flag::Zero => 128,
                Flag::Neg => 64,
                Flag::HalfCarry => 32,
                Flag::Carry => 16
            }
        }
        pub fn set_flags_tri(&mut self,table:[i8;4]){
            let mut flag:u8 = self.F;
            for i in 0..3{
                if table[i]==1{
                    flag = set_bit(flag, (7-i as u8), true) 
                }
                if table[i]==-1{
                    flag = set_bit(flag, 7-i as u8, false) 
                }
            }
            self.F = flag;
        }
        //So to convert a bool, we take 0->-1, 1->1, giving us -1+(2*b)
        //0->keep, -1->unset, 1->set 
        // 0,X -> X
        // 1,X -> 1
        // -1,X -> 0

            

        pub fn process_flags(&mut self, flag_setting:[FlagR;4]){
            match flag_setting[0]{
                FlagR::Keep => (),
                FlagR::Set => self.set_flag(Flag::Zero),
                FlagR::Unset => self.unset_flag(Flag::Zero),
                FlagR::Function(x) => self.flag_cond(Flag::Zero,x())
            }
            match flag_setting[1]{
                FlagR::Keep => (),
                FlagR::Set => self.set_flag(Flag::Neg),
                FlagR::Unset => self.unset_flag(Flag::Neg),
                FlagR::Function(x) => self.flag_cond(Flag::Neg,x())
            }
            match flag_setting[2]{
                FlagR::Keep => (),
                FlagR::Set => self.set_flag(Flag::HalfCarry),
                FlagR::Unset => self.unset_flag(Flag::HalfCarry),
                FlagR::Function(x) => self.flag_cond(Flag::HalfCarry,x())
            }
            match flag_setting[3]{
                FlagR::Keep => (),
                FlagR::Set => self.set_flag(Flag::Carry),
                FlagR::Unset => self.unset_flag(Flag::Carry),
                FlagR::Function(x) => self.flag_cond(Flag::Carry,x())
            }
        }
        pub fn flag_cond(&mut self, flag: Flag,b:bool){
            if b{
                self.set_flag(flag);
            }
            else{
                self.unset_flag(flag);
            }
        }
        pub fn set_flags(&mut self, zero:bool, neg:bool, hc:bool, carry:bool){

            self.F = (((((zero as u8) + ((neg as u8) << 1)) + hc as u8) << 1) + carry as u8)<< 4
        }
        //pub fn dec_hl(&mut self){
         //   change_double_register
        //}
        pub fn unset_flag(&mut self, flag:Flag) {
            self.F &= match flag{
                Flag::Zero => 0x7F,
                Flag::Neg => 0xBF,
                Flag::HalfCarry => 0xDF,
                Flag::Carry => 0xEF
            }
        }
        pub fn increment_pc(&mut self, amount:u8) -> u16{
            self.PC += amount as u16;
            self.PC
        }
        pub fn set_pc(&mut self,amount:u16) -> u16{
            self.PC = amount;
            self.PC
        }
        pub fn increment_pc_one(&mut self) -> u16{
            self.PC += 1;
            self.PC
        }
        pub fn single_reg_code(&mut self,  triplet:u8) ->  SingleReg{
            match triplet{
                0 => SingleReg::B,
                1 => SingleReg::C ,
                2 => SingleReg::D ,
                3 => SingleReg::E ,
                4 => SingleReg::H ,
                5 => SingleReg::L,
                6 => SingleReg::Memptr,
                7 => SingleReg::A 
            }
        }
        pub fn change_single_register(&mut self, reg:SingleReg,fun: &dyn Fn(u8)->u8)->u8{
            self.set_single_register(reg,fun(self.get_register(reg)));
            fun(self.get_register(reg))
        }
        pub fn change_double_register(&mut self, reg:DoubleReg, fun: &dyn Fn(u16)->u16){
            self.set_double_register(reg,fun(self.get_double_register(reg)))
        }
        pub fn apply_fun_to_acc(&mut self, fun: &dyn Fn(u8)->u8){ //Changing this to a single valued anonymous function allows me to handle imms.
            let acc:u8 = self.get_register(SingleReg::A);
            self.set_single_register(SingleReg::A,fun(acc))
        }
        /**pub fn apply_fun_to_reg(&mut self, reg:SingleReg,fun: &dyn Fn(u8)->u8){
            let acc:u8 = self.get_register(SingleReg::A);
            let reg_val:u8 = self.get_register(reg);
            self.set_single_register(SingleReg::A,fun(acc,reg_val))
        }**/
        pub fn r8_op_mid(&mut self, opcode:u8)->SingleReg{
            self.r8_op_end(opcode>>3)
        }
        pub fn r8_op_end(&mut self, opcode:u8)->SingleReg{
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
        pub fn r16_op(&mut self,opcode:u8)->DoubleReg{
            match (opcode >> 4) % 4{
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::SP,
                _=>unreachable!()
            }
        }
        pub fn r16_mem(&mut self,opcode:u8)->DoubleReg{
            match (opcode >> 4) % 4{
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HLP,
                3 => DoubleReg::HLM,
                _=>unreachable!()
            }
        }
        pub fn r16_stk(&mut self,opcode:u8)->DoubleReg{
            match (opcode >> 4) % 4{
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::AF,
                _=>unreachable!()
            }
        }
        pub fn bit_idx(opcode:u8)->u8{
            (opcode >> 3) % 8
        }
        pub fn get_bit(&mut self, reg:SingleReg, idx:u8)->bool{
            return (self.get_register(reg) & (1<<idx)) == (1<<idx)
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
        pub fn reset_flags(&mut self){
            self.set_flags(false,false,false,false);
        }
        pub fn build_registers(mmy:&mut MemoryStruct)->Self{  //Sets the initial states of registers??? 
            return Self{
                A:0, //A IS THE ACCUMULATOR
                B:0,
                C:0,
                D:0,
                E:0, 
                F:0, 
                H:0,
                L:0,
                SP:0, 
                PC:0,
                memory:mmy
            }
        }
    }
}