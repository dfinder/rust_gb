
mod registers { 
    pub fn set_bit(x: u8, idx: u8, b: bool) -> u8 { // should probably make some file that's just helper functions.
        let mask = !(1 << idx);
        let flag = (b as u8) << idx;
        x & mask | flag
    }
    pub fn sign_8(x: u8)-> i8{
        (!x)+1
    }
    pub enum SingleReg{
        A,
        B,
        C,
        D,
        E,
        F,
        H,
        L,
        memptr
    }
    pub enum DoubleReg{
        AF,
        BC,
        DE,
        HL,
        SP,
        PC 
    }
    pub enum Flag{
        Zero,
        Subtraction,
        HalfCarry,
        Carry
    } 
    struct registers{
        A:u8; //A IS THE ACCUMULATOR
        B:u8;
        C:u8;
        D:u8;
        E:u8; 
        F:u8; 
        H:u8;
        L:u8;
        SP:u16; 
        PC:u16;
    }
    impl registers{
        fn get_acc(&mut self) -> u8{
            self.A
        }
        fn set_acc(&mut self, u8 val){
            self.A=val 
        }
        fn get_single_register(&mut self, SingleReg reg)  -> u8{
            match reg{ 
                SingleReg::A => self.A 
                SingleReg::B => self.B 
                SingleReg::C => self.C
                SingleReg::D => self.D
                SingleReg::E => self.E
                SingleReg::F => self.F
                SingleReg::H => self.H
                SingleReg::L => self.L
            }
        }
        fn get_double_register(&mut self, DoubleReg reg) -> u16{
            match reg{ 
                DoubleReg::AF =>  (self.A as u16 ) * 0x80 + self.F
                DoubleReg::BC =>  (self.B as u16 ) * 0x80 + self.C
                DoubleReg::DE =>  (self.D as u16 ) * 0x80 + self.E;
                DoubleReg::HL =>  (self.H as u16 ) * 0x80 + self.L;
            }
        }
        fn get_flag(&mut self, Flag flag) -> bool {
            match flag{
                Flag::Zero => ((self.F & 0x80) == 1)
                Flag::Subtraction => ((self.F & 0x40) == 2)
                Flag::HalfCarry => ((self.F & 0x20) == 4)
                Flag::Carry => ((self.F & 0x10) == 8)
            }
        }
        fn set_single_register(&mut self, SingleReg reg, u8 val){
            match reg{ 
                SingleReg::A => self.A = set 
                SingleReg::B => self.B = set 
                SingleReg::C => self.C = set 
                SingleReg::D => self.D = set 
                SingleReg::E => self.E = set 
                SingleReg::F => panic!("oopsie, we tried to set F");
                SingleReg::H => self.H = set 
                SingleReg::L => self.L = set
            }
        }
        fn set_double_register(&mut self, DoubleReg reg, u16 val) -> u16{
            match reg{ 
                DoubleReg::AF => panic!("We can't set F from Double Set")
                DoubleReg::BC => {
                    self.B = (val >> 8) as u8
                    self.C = val as u8
                }
                DoubleReg::DE => {
                    self.D = (val >> 8) as u8
                    self.E = val as u8
                }
                DoubleReg::HL => {
                {
                    self.H = (val >> 8) as u8
                    self.L = val as u8
                }
                DoubleReg::SP => self.SP=val 
                DoubleReg::PC => self.PC=val 
            }
        }
        fn set_flag(&mut self, Flag flag) -> bool {
            self.F |= match flag{
                Flag::Zero => 128 
                Flag::Subtraction => 64
                Flag::HalfCarry => 32
                Flag::Carry => 16
            }
        }
        fn unset_flag(&mut self, Flag flag) -> bool {
            self.F &= match flag{
                Flag::Zero => 0x7F
                Flag::Subtraction => 0xBF
                Flag::HalfCarry => 0xDF
                Flag::Carry => 0xEF 
            }
        }
        fn increment_pc(&mut self,u8 amount) -> u16{
            self.PC += amount
        }
        fn increment_pc(&mut self) -> u16{
            self.PC += 1
        }
        fn single_reg_code(&mut self, u8 triplet) ->  SingleReg{
            match triplet{
                0 => SingleReg::B
                1 => SingleReg::C 
                2 => SingleReg::D 
                3 => SingleReg::E 
                4 => SingleReg::H 
                5 => SingleReg::L
                6 => SingleReg::memptr
                7 => SingleReg::A 
            }
        }
        fn build_registers()->Self{  //Sets the initial states of registers??? 
            return Self{
                A:0; //A IS THE ACCUMULATOR
                B:0;
                C:0;
                D:0;
                E:0; 
                F:0; 
                H:0;
                L:0;
                SP:0; 
                PC:0;  
            }
        }
}