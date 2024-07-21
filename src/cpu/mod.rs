use std::{thread,time};
mod cpu { 
        const CLOCK_PERIOD: time::duration = Duration::from_nanos(239)
        public struct fun_find{
            mask:u8;
            value:u8;
            function:fn(&mut self)
        }
        public struct cpu{
            registers:register_set;
            memory_reference:memory;
            function_lookup:[fun_find;64];
            cblock_lookup:[fun_find;11];
            current_command:u8;
        }

        public struct function_find
        fn instantiate_cpu() -> Self{
            registers=registers::build_registers()
            memory= memory::build_memory()
            current_command = 0x00; //initalize to a noop
            function_lookup = [
                //Block 1,
                fun_find{0xff,0x00,self.nop},
                fun_find{0xcf,0x01,self.str_r16_imm},
                fun_find{0xcf,0x02,self.str_addr_acc},
                fun_find{0xcf,0x03,self.inc_r16},
                fun_find{0xc7,0x04.self.inc_r8},
                fun_find{0xc7,0x05.self.dec_r8},
                fun_find{0xc7,0x06,self.str_r8_imm}, //note, both loads and stores are ld, so this is ld_r8_imm8
                fun_find{0xff,0x07,self.rcla},
                fun_find{0xff,0x08,self.ld_imm_sp},
                fun_find{0xcf,0x09,self.add_hl},
                fun_find{0xcf,0x0a,self.ld_acc_addr},
                fun_find{0xcf,0x0b,self.dec16},
                fun_find{0xff,0x0f,self.rrca},
                fun_find{0xff,0x1f,self.rra},
                fun_find{0xff,0x2f,self.cpl},
                fun_find{0xff,0x3f,self.ccf},
                fun_find{0xff,0x17,self.cpl},
                fun_find{0xff,0x18,self.jr},
                fun_find{0xE7,0x20,self.jr_cond},
                fun_find{0xff,0x27,self.daa},
                fun_find{0xff,0x37,self.scf},
                fun_find{0xff,0x10,self.stop},
                fun_find{0xff,0x76,self.halt},
                fun_find{0xc0,0x40,self.ld_r8_r8},
                //block2,
                fun_find{0xfc,0x80,self.add_r8},
                fun_find{0xfc,0x88,self.adc_r8},
                fun_find{0xfc,0x90,self.sub_r8},
                fun_find{0xfc,0x98,self.subc_r8},
                fun_find{0xfc,0xa0,self.and_r8},
                fun_find{0xfc,0xa8,self.xor_r8},
                fun_find{0xfc,0xb0,self.or_r8},
                fun_find{0xfc,0xb8,self.cp_r8},
                //block 3,
                fun_find{0xff,0xc6,self.add_r8},
                fun_find{0xff,0xce,self.adc_imm},
                fun_find{0xff,0xd6,self.sub_imm},
                fun_find{0xff,0xde,self.subc_imm},
                fun_find{0xff,0xe6,self.and_imm},
                fun_find{0xff,0xee,self.xor_imm},
                fun_find{0xff,0xf6,self.or_imm},
                fun_find{0xff,0xfe,self.cp_imm},

                fun_find{0xe7,0xc0,self.ret_cond},
                fun_find{0xff,0xc9,self.ret},
                fun_find{0xff,0xd9,self.reti},
                fun_find{0xe7,0xc2,self.jp_cond},
                fun_find{0xff,0xc3,self.jp_imm},
                fun_find{0xff,0xc9,self.jp_hl},
                fun_find{0xe7,0xc4,self.call_cond},
                fun_find{0xff,0xcd,self.call_imm},
                fun_find{0xe7,0xc7,self.rst},
                fun_find{0xcf,0xc1,self.pop},
                fun_find{0xcf,0xc5,self.push},
    
                fun_find{0xff,0xcb,self.cb_block},
                
                fun_find{0xff,0xe2,self.ldh_c},
                fun_find{0xff,0xe0,self.ldh_imm8},
                fun_find{0xff,0xeb,self.ldh_imm16},
                fun_find{0xff,0xf2,self.ldh_c},
                fun_find{0xff,0xf0,self.ldh_imm8},
                fun_find{0xff,0xfb,self.ldh_imm16},

                fun_find{0xff,0xe8,self.add_sp_imm8},
                fun_find{0xff,0xf8,self.ld_hl_imm8},
                fun_find{0xff,0xf8,self.ld_sp_hl},
                fun_find{0xff,0xf3,self.di},
                fun_find{0xff,0xf8,self.ei}
            ]
            self.cb_block_lookup = [
                fun_find{0xf8,0x00,self.rlc_r8},
                fun_find{0xf8,0x08,self.rrc_r8},
                fun_find{0xf8,0x10,self.rl_r8},
                fun_find{0xf8,0x18,self.rr_r8},
                fun_find{0xf8,0x20,self.sla_r8}
                fun_find{0xf8,0x28,self.sra_r8}
                fun_find{0xf8,0x30,self.swap_r8}
                fun_find{0xf8,0x38,self.srl_r8}
                fun_find{0xc0,0x40,self.bit}
                fun_find{0xc0,0x80,self.res}
                fun_find{0xc0,0x90,self.set}
            ]
            
        }
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            let current_command:u8 = self.memory.grab_memory_8(self.register_set.increment_pc());
            //let first_two:u8 = current_command >> 6
            //static masks:[u8]=[0xFF,0xCF,0xE7,0xC0,0xFC, 0xC7,0xF8]
            let taken:bool = false;
            for x in function_lookup{
                if current_command & x.mask == x.value{
                    self.x.function(); //Evaluate for sanity
                    taken=true;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }

        }
        
        fn no_op(&mut self){
            self.wait(1);
        }
        fn rlca(&mut self){ //Rotate register A to the left. Set carry to whatever bit 7 was.
            //self.registers.carry((self.registers.A>>7>0)); 
            //self.registers.A = self.registers.A.rotate_left(1);
            //self.wait(1);
            let carry_value : bool = (self.registers.get_acc() >> 7) > 0
            self.registers.set_flag(registers::Flag::Carry, carry_value)
            self.wait(1)
        }
        fn rla(&mut self){ //Rotate register a to the left _through_ the carry bti .
            let carry:bool = self.registers.get_carry();
            let top:bool = self.registers.A & 0x80 == 0x80
            self.registers.set_carry(top)
            self.registers.A = self.registers.A << 1 + carry
            self.wait(1) 
        }
        fn load_sp(&mut self){
            self.registers.set_double_register(Registers::DoubleReg::SP,self.memory.grab_memory_16(self.registers.increment_pc()));
            self.registers.increment_pc(1);
            self.wait(3);
        }
        fn rrca(&mut self){
            self.registers.set_carry((self.registers.get_acc()<<7)>0)); 
            let acc:u8 = self.registers.get_acc().rotate_right(1);
            self.registers.set_single_register(Registers::SingleRegister::A, acc);
            self.wait(1);
        }
        fn stop(&mut self){
            panic!("crash and burn");
        }
        fn rra(&mut self){ //Need to rewrite.
             //Rotate register a to the right _through_ the carry bti .
                let carry:bool = self.registers.get_carry();
                let bottom:bool = (self.registers.get_acc() & 0x01) == 0x01 //Get bit number 8W
                self.registers.set_carry(bottom)
                self.registers.A = self.registers.A > 1 + carry << 8
                self.wait(1) 
        }
        fn jr(&mut self){
            let next_value: 18 = (self.memory.grab_memory_8(self.registers.increment_pc()) as i8);
            if (next_value>0){
                self.registers.PC += next_value as u8 
                   //but wait, this is signed
            }
            else{
                self.registers.PC -= (!(next_value) + 1) as u8 
            }
            self.wait(3)
        }
        fn str_r16_imm(&mut self){ //Properly LD r16 imm16
            let reg_pair:DoubleReg = match self.current_command{
                0x01 => Registers::BC,
                0x11 => Registers::DE,
                0x21 => Registers::HL,
                0x31 => Registers::SP,
                _ => panic("Failure at mapping for str_r16_imm on line 179")
            }
            self.registers.set_double_register(reg_pair,self.memory.grab_memory_16()) 
            //This may actually also be like... just run str r8 imm twice.
            self.wait(11)
        }
        fn str_addr_acc(&mut self){
            let reg_pair:DoubleReg = match self.current_command{
                0x02 => Registers::DoubleReg::BC,
                0x12 => Registers::DoubleReg::DE,
                0x22 => Registers::DoubleReg::HLP,
                0x32 => Registers::DoubleReg::HLM,
                _ => panic("oof")
            }
            self.memory.set_memory_8(self.registers.get_double_register(reg_pair),self.registers.get_acc())
            self.wait(8)
        }
        fn inc_r8(&mut self){
            let reg_pair:SingleReg = match self.current_command{
                0x04=>Registers::SingleReg::B
                0
            }
            self.wait(4)
        }
        fn inc_r16(&mut self){
            let reg_pair:DoubleReg = match self.current_command{
                0x03 => Registers::DoubleReg::BC,
                0x13 => Registers.DE,
                0x23 => Registers.HL,
                0x33 => Registers.SP
                _ => panic("oof")
            };
            value:u16=self.registers.get_double_register(reg_pair);
            self.registers.set_double_register(reg_pair,value+1);
            self.wait(8)
        }
        fn dec_r16(&mut self){
            let reg_pair:DoubleReg = match self.current_command{
                0x0b => Registers.BC,
                0x1b => Registers.DE,
                0x2b => Registers.HL,
                0x3b => Registers.SP
                _ => panic("oof")
            };
            value:u16=self.registers.get_double_register(reg_pair);
            self.registers.set_double_register(reg_pair,value-1);
            self.wait(8)

        };
        fn cd_block(&mut self){
            self.registers.increment_pc();
            let current_command:u8 = self.memory.grab_memory_8(self.register_set.PC)
            for x in self.cb_block_lookup{
                if current_command & x.mask == x.value{
                    self.x.function(); //Evaluate for sanity
                    taken=true;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }

            //grab the next piece of memory, but we use the CB table.
        }
        fn wait(i8 cycles){
            //4.19 mhz * 4 t cycles 
            thread.sleep(4*CLOCK_PERIOD*cycles);
        }
}