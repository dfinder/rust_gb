pub mod function_table {

    use crate::cpu::cpu::CpuStruct;

    pub type CPUFunct = fn(&mut CpuStruct);
    pub type Alu8Function = fn(&mut CpuStruct, u8, u8) -> u8;

    pub type Alu8SelfFunction = fn(&mut CpuStruct, u8) -> u8;

    pub type Alu16SelfFunction = fn(&mut CpuStruct, u16) -> u16;
    #[derive(Debug, Copy, Clone)]
    pub enum CPUFn {
        Ld8(Src8, Dest8),
        Ld16(Src16, Dest16),
        ALU8(Alu8Function),
        ALU8Self(Alu8SelfFunction),

        ALU16Self(Alu16SelfFunction),
        Other(CPUFunct),
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Src16 {
        Imm16,
        HL,
        SP,
    }
    #[derive(Debug, Copy, Clone)]
    pub enum Dest16 {
        R16,
        PC,
        SP,
        HL,
    }
    #[derive(Debug, Copy, Clone)]
    pub enum Src8 {
        Imm16Mem,
        Imm8,
        HighBank,
        Acc,
        R8Mid,
        R8,
        HighC,
        R16Mem,
    }
    #[derive(Debug, Copy, Clone)]
    pub enum Dest8 {
        R8,
        Imm16Mem,
        Imm8High,
        Acc,
        R16Mem,
        HighC,
    }
    //#[derive(Copy)]
    #[derive(Debug, Copy, Clone)]
    pub struct FunFind {
        pub mask: u8,
        pub value: u8,
        pub function: CPUFn, //,argument Arg. returns false if we have a longer wait.
        pub wait: u8,
        //flags: FlagS,
        //bytes: u8//1,2,3, measures the enums.
    }

    impl FunFind {
        pub fn fun_find(mask: u8, value: u8, function: CPUFn, wait: u8) -> Self {
            Self {
                mask,
                value,
                function,
                wait,
            }
        }
        pub fn function_lookup() -> [FunFind; 63] {
            return [
                //Block 1,
                FunFind::fun_find(0xff, 0x00, CPUFn::Other(CpuStruct::nop), 1), //done //Reasses waits.
                FunFind::fun_find(0xcf, 0x01, CPUFn::Ld16(Src16::Imm16, Dest16::R16), 3), //done
                FunFind::fun_find(0xcf, 0x02, CPUFn::Ld8(Src8::Acc, Dest8::R16Mem), 2),
                FunFind::fun_find(0xcf, 0x03, CPUFn::ALU16Self(CpuStruct::inc_r16), 2), //done
                FunFind::fun_find(0xc7, 0x04, CPUFn::Other(CpuStruct::inc_r8), 0),   //done
                FunFind::fun_find(0xc7, 0x05, CPUFn::Other(CpuStruct::dec_r8), 0),   //done
                FunFind::fun_find(0xc7, 0x06, CPUFn::Ld8(Src8::Imm8, Dest8::R8), 2),    //done
                FunFind::fun_find(0xff, 0x07, CPUFn::ALU8Self(CpuStruct::rlc), 1),      //done
                FunFind::fun_find(0xff, 0x08, CPUFn::Ld16(Src16::Imm16, Dest16::SP), 1), //done
                FunFind::fun_find(0xcf, 0x09, CPUFn::Other(CpuStruct::add_hl), 2),      //
                FunFind::fun_find(0xcf, 0x0a, CPUFn::Ld8(Src8::R16Mem, Dest8::Acc), 1), //done
                FunFind::fun_find(0xcf, 0x0b, CPUFn::ALU16Self(CpuStruct::dec_r16), 2), //done
                FunFind::fun_find(0xff, 0x0f, CPUFn::ALU8Self(CpuStruct::rrc), 1),      //done
                FunFind::fun_find(0xff, 0x1f, CPUFn::ALU8Self(CpuStruct::rr), 1),       //done
                FunFind::fun_find(0xff, 0x2f, CPUFn::Other(CpuStruct::cpl), 1),         //done
                FunFind::fun_find(0xff, 0x3f, CPUFn::Other(CpuStruct::ccf), 1),         //done
                FunFind::fun_find(0xff, 0x17, CPUFn::ALU8Self(CpuStruct::rl), 1),       //done
                FunFind::fun_find(0xff, 0x18, CPUFn::Other(CpuStruct::jr_imm), 3), //?
                FunFind::fun_find(0xe7, 0x20, CPUFn::Other(CpuStruct::jr_cond), 3),     //
                FunFind::fun_find(0xff, 0x27, CPUFn::ALU8Self(CpuStruct::daa), 1),      //?
                FunFind::fun_find(0xff, 0x37, CPUFn::Other(CpuStruct::scf), 1),         //done
                FunFind::fun_find(0xff, 0x10, CPUFn::Other(CpuStruct::stop), 0),        //?
                FunFind::fun_find(0xff, 0x76, CPUFn::Other(CpuStruct::halt), 0),        //?
                FunFind::fun_find(0xc0, 0x40, CPUFn::Ld8(Src8::R8, Dest8::R8), 1),
                FunFind::fun_find(0xf8, 0x80, CPUFn::ALU8(CpuStruct::add), 0),
                FunFind::fun_find(0xf8, 0x88, CPUFn::ALU8(CpuStruct::adc), 0),
                FunFind::fun_find(0xf8, 0x90, CPUFn::ALU8(CpuStruct::sub), 0),
                FunFind::fun_find(0xf8, 0x98, CPUFn::ALU8(CpuStruct::subc), 0),
                FunFind::fun_find(0xf8, 0xa0, CPUFn::ALU8(CpuStruct::and), 0),
                FunFind::fun_find(0xf8, 0xa8, CPUFn::ALU8(CpuStruct::xor), 1),
                FunFind::fun_find(0xf8, 0xb0, CPUFn::ALU8(CpuStruct::or), 1),
                FunFind::fun_find(0xf8, 0xb8, CPUFn::ALU8(CpuStruct::cp), 1),
                FunFind::fun_find(0xff, 0xc6, CPUFn::ALU8(CpuStruct::add), 1),
                FunFind::fun_find(0xff, 0xce, CPUFn::ALU8(CpuStruct::adc), 1),
                FunFind::fun_find(0xff, 0xd6, CPUFn::ALU8(CpuStruct::sub), 1),
                FunFind::fun_find(0xff, 0xde, CPUFn::ALU8(CpuStruct::subc), 1),
                FunFind::fun_find(0xff, 0xe6, CPUFn::ALU8(CpuStruct::and), 1),
                FunFind::fun_find(0xff, 0xee, CPUFn::ALU8(CpuStruct::xor), 1),
                FunFind::fun_find(0xff, 0xf6, CPUFn::ALU8(CpuStruct::or), 1),
                FunFind::fun_find(0xff, 0xfe, CPUFn::ALU8(CpuStruct::cp), 1),
                FunFind::fun_find(0xe7, 0xc0, CPUFn::Other(CpuStruct::ret_cond), 5),
                FunFind::fun_find(0xff, 0xc9, CPUFn::Other(CpuStruct::ret), 4),
                FunFind::fun_find(0xff, 0xd9, CPUFn::Other(CpuStruct::reti), 4),
                FunFind::fun_find(0xe7, 0xc2, CPUFn::Other(CpuStruct::jp_cond_imm), 3,),
                FunFind::fun_find(0xff, 0xc3, CPUFn::Ld16(Src16::Imm16, Dest16::PC), 4),
                FunFind::fun_find(0xff, 0xe9, CPUFn::Ld16(Src16::HL, Dest16::PC), 1),
                FunFind::fun_find(0xe7, 0xc4, CPUFn::Other(CpuStruct::call_cond), 6),
                FunFind::fun_find(0xff, 0xcd, CPUFn::Other(CpuStruct::call_imm), 6),
                FunFind::fun_find(0xc7, 0xc7, CPUFn::Other(CpuStruct::rst), 4),
                FunFind::fun_find(0xcf, 0xc1, CPUFn::Other(CpuStruct::pop), 3),
                FunFind::fun_find(0xcf, 0xc5, CPUFn::Other(CpuStruct::push), 4),
                FunFind::fun_find(0xff, 0xcb, CPUFn::Other(CpuStruct::cb_block), 1),
                FunFind::fun_find(0xff, 0xe2, CPUFn::Ld8(Src8::Acc, Dest8::HighC), 1), //We're storing if we're mapping to memory, we're loading if we'r
                FunFind::fun_find(0xff, 0xe0, CPUFn::Ld8(Src8::Acc, Dest8::Imm8High), 2),
                FunFind::fun_find(0xff, 0xea, CPUFn::Ld8(Src8::Acc, Dest8::Imm16Mem), 3),
                FunFind::fun_find(0xff, 0xf0, CPUFn::Ld8(Src8::HighBank, Dest8::Acc), 2),
                FunFind::fun_find(0xff, 0xf2, CPUFn::Ld8(Src8::HighC, Dest8::Acc), 1),
                FunFind::fun_find(0xff, 0xfa, CPUFn::Ld8(Src8::Imm16Mem, Dest8::Acc), 3),
                FunFind::fun_find(0xff, 0xe8, CPUFn::Other(CpuStruct::add_sp_imm8), 4), //
                FunFind::fun_find(0xff, 0xf8, CPUFn::Other(CpuStruct::ld_hl_imm8), 3),
                FunFind::fun_find(0xff, 0xf9, CPUFn::Ld16(Src16::SP, Dest16::HL), 2),
                FunFind::fun_find(0xff, 0xf3, CPUFn::Other(CpuStruct::di), 1),
                FunFind::fun_find(0xff, 0xfb, CPUFn::Other(CpuStruct::ei), 1),
            ];
        }
        /* pub fn function_lookup() -> [FunFind; 63] {
                   return [
                       //Block 1,
                       FunFind::fun_find(0xff, 0x00, CpuStruct::nop, 1), //done //Reasses waits.
                       FunFind::fun_find(0xcf, 0x01, CpuStruct::ldi_r16, 3), //done
                       FunFind::fun_find(0xcf, 0x02, CpuStruct::str_acc_rmem, 2),
                       FunFind::fun_find(0xcf, 0x03, CpuStruct::inc_r16, 2), //done
                       FunFind::fun_find(0xc7, 0x04, CpuStruct::inc_r8, 1),  //done
                       FunFind::fun_find(0xc7, 0x05, CpuStruct::dec_r8, 1),  //done
                       FunFind::fun_find(0xc7, 0x06, CpuStruct::ldi_r8, 2),  //done
                       FunFind::fun_find(0xff, 0x07, CpuStruct::rlc, 1),     //done
                       FunFind::fun_find(0xff, 0x08, CpuStruct::ld_imm_sp, 5), //done
                       FunFind::fun_find(0xcf, 0x09, CpuStruct::add_hl, 2),  //
                       FunFind::fun_find(0xcf, 0x0a, CpuStruct::ld_acc_addr, 5), //done
                       FunFind::fun_find(0xcf, 0x0b, CpuStruct::dec_r16, 2), //done
                       FunFind::fun_find(0xff, 0x0f, CpuStruct::rrc, 1),     //done
                       FunFind::fun_find(0xff, 0x1f, CpuStruct::rr, 1),      //done
                       FunFind::fun_find(0xff, 0x2f, CpuStruct::cpl, 1),     //done
                       FunFind::fun_find(0xff, 0x3f, CpuStruct::ccf, 1),     //done
                       FunFind::fun_find(0xff, 0x17, CpuStruct::rl, 1),      //done
                       FunFind::fun_find(0xff, 0x18, CpuStruct::jr_imm, 3),  //?
                       FunFind::fun_find_w(0xe7, 0x20, CpuStruct::jr_cond, 3, 5), //
                       FunFind::fun_find(0xff, 0x27, CpuStruct::daa, 1),     //?
                       FunFind::fun_find(0xff, 0x37, CpuStruct::scf, 1),     //done
                       FunFind::fun_find(0xff, 0x10, CpuStruct::stop, 0),    //?
                       FunFind::fun_find(0xff, 0x76, CpuStruct::halt, 0),    //?
                       FunFind::fun_find_w(0xc0, 0x40, CpuStruct::ld_r8_r8, 1, 2),
                       FunFind::fun_find_w(0xf8, 0x80, CpuStruct::add, 1, 2),
                       FunFind::fun_find_w(0xf8, 0x88, CpuStruct::adc, 1, 2),
                       FunFind::fun_find_w(0xf8, 0x90, CpuStruct::sub, 1, 2),
                       FunFind::fun_find_w(0xf8, 0x98, CpuStruct::subc, 1, 2),
                       FunFind::fun_find_w(0xf8, 0xa0, CpuStruct::and, 1, 2),
                       FunFind::fun_find_w(0xf8, 0xa8, CpuStruct::xor, 1, 2),
                       FunFind::fun_find_w(0xf8, 0xb0, CpuStruct::or, 1, 2),
                       FunFind::fun_find_w(0xf8, 0xb8, CpuStruct::cp, 1, 2),
                       FunFind::fun_find_w(0xff, 0xc6, CpuStruct::add, 1, 2),
                       FunFind::fun_find_w(0xff, 0xce, CpuStruct::adc, 1, 2),
                       FunFind::fun_find_w(0xff, 0xd6, CpuStruct::sub, 1, 2),
                       FunFind::fun_find_w(0xff, 0xde, CpuStruct::subc, 1, 2),
                       FunFind::fun_find_w(0xff, 0xe6, CpuStruct::and, 1, 2),
                       FunFind::fun_find_w(0xff, 0xee, CpuStruct::xor, 1, 2),
                       FunFind::fun_find_w(0xff, 0xf6, CpuStruct::or, 1, 2),
                       FunFind::fun_find_w(0xff, 0xfe, CpuStruct::cp, 1, 2),
                       FunFind::fun_find_w(0xe7, 0xc0, CpuStruct::ret_cond, 5, 2),
                       FunFind::fun_find(0xff, 0xc9, CpuStruct::ret, 4),
                       FunFind::fun_find(0xff, 0xd9, CpuStruct::reti, 4),
                       FunFind::fun_find_w(0xe7, 0xc2, CpuStruct::jp_cond_imm, 3, 2),
                       FunFind::fun_find(0xff, 0xc3, CpuStruct::jp_imm, 4),
                       FunFind::fun_find(0xff, 0xe9, CpuStruct::jp_hl, 1),
                       FunFind::fun_find_w(0xe7, 0xc4, CpuStruct::call_cond, 6, 3),
                       FunFind::fun_find_w(0xff, 0xcd, CpuStruct::call_imm, 6, 3),
                       FunFind::fun_find(0xc7, 0xc7, CpuStruct::rst, 4),
                       FunFind::fun_find(0xcf, 0xc1, CpuStruct::pop, 3),
                       FunFind::fun_find(0xcf, 0xc5, CpuStruct::push, 4),
                       FunFind::fun_find(0xff, 0xcb, CpuStruct::cb_block, 1),
                       FunFind::fun_find(0xff, 0xe2, CpuStruct::str_c, 1), //We're storing if we're mapping to memory, we're loading if we'r
                       FunFind::fun_find(0xff, 0xe0, CpuStruct::str_imm8, 2),
                       FunFind::fun_find(0xff, 0xea, CpuStruct::str_imm16, 3),
                       FunFind::fun_find(0xff, 0xf0, CpuStruct::ld_imm8, 2),
                       FunFind::fun_find(0xff, 0xf2, CpuStruct::ld_c, 1),
                       FunFind::fun_find(0xff, 0xfa, CpuStruct::ld_imm16, 3),
                       FunFind::fun_find(0xff, 0xe8, CpuStruct::add_sp_imm8, 4), //
                       FunFind::fun_find(0xff, 0xf8, CpuStruct::ld_hl_imm8, 3),
                       FunFind::fun_find(0xff, 0xf9, CpuStruct::ld_sp_hl, 2),
                       FunFind::fun_find(0xff, 0xf3, CpuStruct::di, 1),
                       FunFind::fun_find(0xff, 0xfb, CpuStruct::ei, 1),
                   ];
               }
        */
        pub fn cb_block_lookup() -> [FunFind; 11] {
            return [
                FunFind::fun_find(0xf8, 0x00, CPUFn::ALU8Self(CpuStruct::rlc), 2),
                FunFind::fun_find(0xf8, 0x08, CPUFn::ALU8Self(CpuStruct::rrc), 2),
                FunFind::fun_find(0xf8, 0x10, CPUFn::ALU8Self(CpuStruct::rl), 2),
                FunFind::fun_find(0xf8, 0x18, CPUFn::ALU8Self(CpuStruct::rr), 2),
                FunFind::fun_find(0xf8, 0x20, CPUFn::ALU8Self(CpuStruct::sla), 2),
                FunFind::fun_find(0xf8, 0x28, CPUFn::ALU8Self(CpuStruct::sra), 2),
                FunFind::fun_find(0xf8, 0x30, CPUFn::ALU8Self(CpuStruct::swap), 2),
                FunFind::fun_find(0xf8, 0x38, CPUFn::ALU8Self(CpuStruct::srl), 2),
                FunFind::fun_find(0xc0, 0x40, CPUFn::ALU8Self(CpuStruct::bit), 2),
                FunFind::fun_find(0xc0, 0x80, CPUFn::ALU8Self(CpuStruct::res), 2),
                FunFind::fun_find(0xc0, 0x90, CPUFn::ALU8Self(CpuStruct::set), 2),
            ];
        }
    }
}
