use std::{thread,time};
mod cpu { 
        const CLOCK_PERIOD: time::duration = Duration::from_nanos(239)
        public struct fun_find{
            mask:u8;
            value:u8;
            function:&dyn Fn(&mut self,arg)->bool;//,argument Arg. returns false if we have a longer wait.
            //What if we had this as self, input, output, 
            wait:u8;
            wait_cond:Optional<u8>;
            flags: FlagS;
            bytes: u8;//1,2,3, measures the enums.
        }
        enum FlagR{
            Set,
            Unset,
            Keep, 
            Function(&dyn Fn()->bool) //What should the argument here be?
        }
        enum FlagT{
            ClearAll,
            KeepAll,
            Specific([FlagRelation;4]) //Nah this is dumb.
        }
        enum InterruptState{
            Enabled;
            AlmostEnabled;
            AlmostDisabled;
            Disabled;
        }
        enum Arg{
            SingleReg(Registers::SingleReg),
            DoubleReg(Registers::DoubleReg),
            //PairSingleReg(Registers::SingleReg,Registers:SingleReg)
            MemReg(Registers::DoubleReg),
            StackReg(Registers::DoubleReg),
            Con(bool),
            Imm8(u8),
            Imm16(u16),
            Arg_Nest(Box<Arg>,Box<Arg>)
            Empty
        }
        public struct cpu{
            registers:register_set;
            memory_reference:memory;
            function_lookup:[fun_find;64];
            cblock_lookup:[fun_find;11];
            current_command:u8;
            //used for mem reads to HL, failed conditional jumps
            //argument:Argument;
        }
        fn instantiate_cpu() -> Self{
            registers=registers::build_registers()
            memory= memory::build_memory()
            current_command = 0x00; //initalize to a noop
            function_lookup = [
                //Block 1,
                fun_find{0xff,0x00,self.nop,1,Empty}, //!
                fun_find{0xcf,0x01,self.str_r16_imm,3,Empty},  //!
                fun_find{0xcf,0x02,self.str_addr_acc,2,Empty}, //!
                fun_find{0xcf,0x03,self.inc_r16,2,Empty},//!
                fun_find{0xc7,0x04.self.inc_r8,1,Empty},//!
                fun_find{0xc7,0x05.self.dec_r8,1,Empty},//!
                fun_find{0xc7,0x06,self.str_r8_imm,2,Empty}, //note, both loads and stores are ld, so this is ld_r8_imm8
                fun_find{0xff,0x07,self.rlca,1,Empty},//!
                fun_find{0xff,0x08,self.ld_imm_sp,4,Empty},//!
                fun_find{0xcf,0x09,self.add_hl,2,Empty},//
                fun_find{0xcf,0x0a,self.ld_acc_addr,5,Empty},//!
                fun_find{0xcf,0x0b,self.dec16,2,Empty},//!
                fun_find{0xff,0x0f,self.rrca,1,Empty},//!
                fun_find{0xff,0x1f,self.rra,1,Empty},//!
                fun_find{0xff,0x2f,self.cpl,1,None},//!
                fun_find{0xff,0x3f,self.ccf,1,None},//!
                fun_find{0xff,0x17,self.rla,1,None},//!
                fun_find{0xff,0x18,self.jr_imm},//?
                fun_find{0xe7,0x20,self.jr_cond},//
                fun_find{0xff,0x27,self.daa},//?
                fun_find{0xff,0x37,self.scf,1,None},//!
                fun_find{0xff,0x10,self.stop},//?   
                fun_find{0xff,0x76,self.halt},//?
                fun_find{0xc0,0x40,self.ld_r8_r8,1,Some(2)},
                //block2,
                fun_find{0xf8,0x80,self.add_r8,1,None},
                fun_find{0xf8,0x88,self.adc_r8},
                fun_find{0xf8,0x90,self.sub_r8},
                fun_find{0xf8,0x98,self.subc_r8},
                fun_find{0xf8,0xa0,self.and_r8},
                fun_find{0xf8,0xa8,self.xor_r8},
                fun_find{0xf8,0xb0,self.or_r8},
                fun_find{0xf8,0xb8,self.cp_r8},
                //block 3,
                fun_find{0xff,0xc6,self.add_imm},
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
                fun_find{0xe7,0xc2,self.jp_cond_imm},
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
                fun_find{0xf8,0x00,self.rlc_r8,2,Empty},
                fun_find{0xf8,0x08,self.rrc_r8,2,Empty},
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
        fn get_double_register_from_opcode(){

        }
        fn process_single_arg(Arg arg)->SingleReg{
            reg = match arg{
                SingleReg(reg) => reg
                _ => !unreachable()
            }
            reg 
        }
        fn get_arg_val(Arg arg){ //Holy mother of polymorphism

        }
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            let current_command:u8 = self.memory.grab_memory_8(self.registers.increment_pc());
            //let first_two:u8 = current_command >> 6
            //static masks:[u8]=[0xFF,0xCF,0xE7,0xC0,0xC7,0xF8]
            let mut taken:bool = false;
            for x in function_lookup{
                if current_command & x.mask == x.value{
                    let argument:Argument = match x.mask{
                        0xff => Arg::Empty
                        0xcf => get_double_register_from_opcode(current_command)
                        0xe7 => Arg::Cond(self.registers.get_cond(current_command))
                        0xc0 => Arg::Pair(self.registers.r8_op_mid(),self.registers.r8_op_end())
                        0xc7 => Arg::
                        0xf8 => r8_end
                
                        let reg_pair:DoubleReg = self.registers.r16_op(self.current_command);
                    }**/
                    let mut result:bool = self.x.function(argument)
                    if self.x.function(){
                        self.wait(x.wait);

                    }else{
                        self.wait(x.cond_wait);
                    }; //Evaluate for sanity

                    taken=true;
                    break;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }
            //self.registers.increment_pc()

        }
        
        fn no_op(&mut self)->bool{
            true
        }

        fn ld_imm_sp(&mut self){
            self.registers.set_double_register(Registers::DoubleReg::SP,self.memory.grab_memory_16(self.registers.increment_pc()));
            self.registers.increment_pc(1);
        }
        fn rrc_r8(&mut self, Arg arg)->bool{
            let arg_reg = match arg{
                SingleReg(reg) => reg
                _ => !unreachable()
            }
            self.registers.set_carry(self.registers.get_bit(arg_reg, 0)); //Rotate right
            self.registers.change_single_register(arg_reg, &|x| x.rotate_right(1));
        }
        fn rrca(&mut self,Arg arg)->bool{
            self.rrc_r8(Arg::SingleReg(Registers::SingleReg::A))
        }
        fn rlc_r8(&mut self, Arg arg) -> bool{
            let arg_reg = match arg{
                SingleReg(reg) => reg
                _ => !unreachable()
            }
            self.registers.set_carry(self.registers.get_bit(arg_reg, 7)); //Rotate right
            self.registers.change_single_register(arg_reg, &|x| x.rotate_left(1));
        }
        fn rlca(&mut self, Arg arg)->bool{
            self.rlc_r8(Arg::SingleReg(Registers::SingleReg::A));
        }
        fn rla(&mut self){ //Rotate register a to the left _through_ the carry bti .
            let carry:bool = self.registers.get_carry();
            let top:bool = self.registers.get_bit(Registres::SingleReg::A,7);
            self.registers.set_carry(top);
            self.change_single_register(Registers::SingleReg::A, &|x| x<<1 + carry); //Check to see if I need wrapping left shi
            self.wait(1);
        }
        fn rra(&mut self,Arg arg)->bool{ //Need to rewrite.
            let carry:bool = self.registers.get_carry();
            let bottom:bool = self.registers.get_bit(Registers::SingleReg::A, 0)
            self.registers.set_carry(bottom)
            self.registers.change_single_register(Registers::SingleReg::A, &|x| x>>1 + carry<<7)
            self.wait(1) 
        }
        fn jr_imm(&mut self)->bool{
            let next_value: 18 = (self.memory.grab_memory_8(self.registers.increment_pc()) as i8);
            self.registers.change_double_register(registers::DoubleReg::PC, &|x| x.wrapping_add(next_value);
            self.wait(3)
        }
        fn jr_cond(&mut self, Arg arg)->bool{
            if !self.registers.get_cond(self.current_command){
                false
            }
            else{
                self.jr_imm()
            }
        }
        fn daa(&mut self, Arg arg)->bool{
            let subtract = registers.get_flag_bit(registers::Flag::Neg);
            let hcarry = registers.get_flag_bit(registers::Flag::HalfCarry);
            let carry = registers.get_flag_bit(registers::Flag::Carry);
            
            //To complete
        }

        fn str_r16_imm(&mut self, Arg arg)->bool{ //Properly LD r16 imm16
            let reg_pair:DoubleReg = self.registers.r16_op(self.current_command);
            self.registers.set_double_register(reg_pair,self.memory.grab_memory_16()) 
            //This may actually also be like... just run str r8 imm twice.
            self.wait(3)
        }
        fn str_addr_acc(&mut self, Arg arg)->bool{
            let reg_pair:DoubleReg = self.registers.r16_mem(self.current_command);
            self.memory.set_memory_8(self.registers.get_double_register(reg_pair),self.registers.get_acc())
        }
        fn inc_r8(&mut self, Arg arg)->bool{
            let reg:SingleReg = match arg{
                SingleReg(rg)=>rg;
                _ => !unreachable()
            } //self.registers.r8_op_mid(self.current_command);
            self.registers.change_single_register(reg, &|x| x+1);
            reg != Registers::SingleReg::memptr
        }
        fn dec_r8(&mut self, Arg arg)->bool{
            let reg:SingleReg = self.registers.r8_op_mid(self.current_command);
            if reg == Registers::SingleReg::memptr{
                self.wait(2);
            }
            self.registers.change_single_register(reg, &|x| x-1);
            self.wait(1)
        }
        fn inc_r16(&mut self, Arg arg)->bool{
            let reg_pair:DoubleReg = self.registers.r16_op(self.current_command);
            self.registers.change_double_register(reg_pair,&|x| x+1);
        }
        fn dec_r16(&mut self, Arg arg)->bool{
            let reg_pair:DoubleReg = self.registers.r16_op(self.current_command);
            self.registers.change_double_register(reg_pair,&|x| x-1);
        };
        fn str_r8_imm(&mut self, Arg arg)->bool{
            let reg:SingleReg = self.registers.r8_op_mid(self.current_command);
            let imm:u8 = self.memory.grab_memory_8(self.registers.increment_pc())
            self.registers.set_single_register(reg,imm)
        }
        fn ld_imm_sp(&mut self, Arg arg)->bool{
            self.memory.grab_memory_16(self.registers.increment_pc());
            self.registers.increment_pc()
            self.memory.set_memory_16(self.registers.get_double_register(Registers::DoubleReg::SP))
        }
        fn ld_r8_r8(&mut self, Arg arg ){
            let reg_dest, reg_src, = match arg{
                Arg::PairSingleReg(reg1,reg2) => reg1,reg2
                _ => !unreachable()
            }
            self.registers.set_single_register(reg_dest,self.registers.get_register(reg_src));
        
            return reg_src != Registers::SingleReg::memptr && reg_dest != Registers::SingleReg::memptr
        }
        fn ld_acc_addr(&mut self, Arg arg)->bool{ //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg:DoubleReg = self.registers.r16_mem(self.current_command);
            self.set_acc(self.memory.grab_memory_8(reg))
            if reg == DoubleReg::HLP || reg == DoubleReg::HLM{
                self.registers.set_double_register(reg,0);
            }
        }
        fn add_hl(&mut self, Arg arg)->bool{
            let reg_pair:DoubleReg = self.registers.r16_op(self.current_command);
            let double_reg_val:u16 = self.registers.get_double_register(reg_pair);
            self.registers.change_double_register(Register::DoubleReg::HL, &|x|x+double_reg_val));
            //TODO: Flags
        }

        fn cpl(&mut self, Arg arg)->bool{ //Invert A
            self.registers.change_single_register(Registers::SingleReg::A,&|x| !a)
            self.registers.set_flag(Registers::Flags::HalfCarry)
            self.registers.set_flag(Registers::Flags::Neg)
        }
        fn ccf(&mut self, Arg arg)->bool{
            self.registers.flip_carry();
            true
        }
        fn scf(&mut self, Arg arg)->bool{
            self.registers.set_flag(Registers::Flags::Carry);
        }
        fn add_r8(&mut self, Arg arg)->bool{ 
            self.registers.apply_fun_to_acc(arg_reg, &|x|x+self.registers.get_register(process_single_arg(arg)))
        } 
        fn adc_r8(&mut self, Arg arg)->bool{
            let arg_reg = process_single_arg();
            let carry = self.registers.get_flag(Registers::Flag::Carry)
            self.registers.apply_fun_to_acc(arg_reg, &|x|x+carry+self.registers.get_register(arg_reg))
        } 
        fn sub_r8(&mut self, Arg arg)->bool{
            self.registers.apply_fun_to_acc(arg_reg, &|x|x-self.registers.get_register(process_single_arg(arg)))
        }
        fn subc_r8(&mut self, Arg arg)->bool{
            let arg_reg = process_single_arg();
            let carry = self.registers.get_flag(Registers::Flag::Carry);
            self.registers.apply_fun_to_acc(arg_reg, &|x| x+carry+self.registers.get_register(arg_reg))
        } 
        fn and_r8(&mut self, Arg arg)->bool{
            self.registers.apply_fun_to_acc(arg_reg, &|x| x&self.registers.get_register(process_single_arg(arg)))
        } 
        fn or_r8(&mut self, Arg arg)->bool{
            self.registers.apply_fun_to_acc(arg_reg, &|x| x|self.registers.get_register(process_single_arg(arg)))
        }
        fn cp_r8(&mut self, Arg arg)->bool{
            let arg_reg = get_arg_val_single(arg);
            let acc = self.registers.get_register(Registers::SingleReg::A);

            self.registers.set_flags((arg_reg-acc == 0),true,(#Figure out the half carry flag ),(arg_reg>a))
            //self.registers.apply_fun_to_acc(arg_reg, &|x|x&self.registers.get_register(process_single_arg(arg)))
        } 
        fn add_imm(&mut self, Arg arg)->bool{
        }
        fn adc_imm(&mut self, Arg arg)->bool{
        }
        fn sub_imm(&mut self, Arg arg)->bool{
        }
        fn subc_imm(&mut self, Arg arg)->bool{
        }
        fn and_imm(&mut self, Arg arg)->bool{
        }
        fn or_imm(&mut self, Arg arg)->bool{
        }
        fn cp_imm(&mut self, Arg arg)->bool{
        }
        fn ret_cond(&mut self, Arg arg)->bool{
        } 
        fn ret(&mut self, Arg arg)->bool{
        }
        fn reti(&mut self, Arg arg)->bool{
        } 
        fn jp_cond_imm(&mut self, Arg arg)->bool{
        } 
        fn jp_imm(&mut self, Arg arg)->bool{
        } 
        fn jp_hl(&mut self, Arg arg)->bool{
        } 
        fn call_cond(&mut self, Arg arg)->bool{
        } 
        fn call_imm(&mut self, Arg arg)->bool{
        } 
        fn rst(&mut self, Arg arg)->bool{
        }
        fn pop(&mut self, Arg arg)->bool{
        }
        fn push(&mut self, Arg arg)->bool{
        }
        fn ldh_c(&mut self, Arg arg)->bool{
        } 
        fn ldh_imm8(&mut self, Arg arg)->bool{
        }
        fn ldh_imm16(&mut self, Arg arg)->bool{
        }
        fn ldh_c(&mut self, Arg arg)->bool{
        } 
        fn ldh_imm8(&mut self, Arg arg)->bool{
        } 
        fn ldh_imm16(&mut self, Arg arg)->bool{
        }
        fn add_sp_imm8(&mut self, Arg arg)->bool{
        }
        fn ld_hl_imm8(&mut self, Arg arg)->bool{
        }
        fn ld_sp_hl(&mut self, Arg arg)->bool{
        }
        fn di(&mut self, Arg arg)->bool{
        }
        fn ei(&mut self, Arg arg)->bool{
        } 
        fn rlc_r8(&mut self, Arg arg)->bool{
        }
        fn rl_r8(&mut self, Arg arg)->bool{
        }
        fn rr_r8(&mut self, Arg arg)->bool{
        } 
        fn sla_r8(&mut self, Arg arg)->bool{
        } 
        fn sra_r8(&mut self, Arg arg)->bool{
        } 
        fn swap_r8(&mut self, Arg arg)->bool{
        } 
        fn srl_r8(&mut self, Arg arg)->bool{
        } 
        fn bit(&mut self, Arg arg)->bool{
        } 
        fn res(&mut self, Arg arg)->bool{
        } 
        fn set(&mut self, Arg arg)->bool{
        }
        fn stop(&mut self, Arg arg)->bool{
            loop {
                self.wait(1);
                //PAUSE THE GPU
                //BREAK IF BUTTON PRESSED.
            }
        }
        fn halt(&mut self, arg)->bool{
            loop { 
                self.wait(1) //Enter low power mode until an interrupt
            }
        }
        fn cd_block(&mut self){
            let current_command:u8 = self.memory.grab_memory_8(self.registers.increment_pc())
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