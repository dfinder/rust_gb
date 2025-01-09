pub mod function_table {
    use crate::cpu::cpu::{CPUFunct, CpuStruct};
    //#[derive(Copy)]
    #[derive(Debug)]
    pub struct FunFind {
        pub mask: u8,
        pub value: u8,
        pub function: CPUFunct, //,argument Arg. returns false if we have a longer wait.
        pub wait: u8,
        pub wait_cond: Option<u8>,
        //flags: FlagS,
        //bytes: u8//1,2,3, measures the enums.
    }
    impl FunFind {
        pub fn fun_find(mask: u8, value: u8, function: CPUFunct, wait: u8) -> Self {
            Self {
                mask,
                value,
                function,
                wait,
                wait_cond: None,
            }
        }
        pub fn fun_find_w(
            mask: u8,
            value: u8,
            function: CPUFunct,
            wait: u8,
            wait_cond: u8,
        ) -> Self {
            Self {
                mask: mask,
                value: value,
                function: function,
                wait: wait,
                wait_cond: Some(wait_cond),
            }
        }
        pub fn function_lookup() -> [FunFind; 63] {
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
                FunFind::fun_find(0xff, 0xe2, CpuStruct::str_c, 1), //We're storing if we're mapping to memory, we're loadingif we'r
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
        pub fn cb_block_lookup() -> [FunFind; 11] {
            return [
                FunFind::fun_find_w(0xf8, 0x00, CpuStruct::rlc, 2, 4),
                FunFind::fun_find_w(0xf8, 0x08, CpuStruct::rrc, 2, 4),
                FunFind::fun_find_w(0xf8, 0x10, CpuStruct::rl, 2, 4),
                FunFind::fun_find_w(0xf8, 0x18, CpuStruct::rr, 2, 4),
                FunFind::fun_find_w(0xf8, 0x20, CpuStruct::sla, 2, 4),
                FunFind::fun_find_w(0xf8, 0x28, CpuStruct::sra, 2, 4),
                FunFind::fun_find_w(0xf8, 0x30, CpuStruct::swap, 2, 4),
                FunFind::fun_find_w(0xf8, 0x38, CpuStruct::srl, 2, 4),
                FunFind::fun_find_w(0xc0, 0x40, CpuStruct::bit, 2, 3),
                FunFind::fun_find_w(0xc0, 0x80, CpuStruct::res, 2, 4),
                FunFind::fun_find_w(0xc0, 0x90, CpuStruct::set, 2, 4),
            ];
        }
    }
}
