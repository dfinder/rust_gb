

pub mod cpu { 
    use crate::registers::registers::{self, SingleReg};
    use crate::registers::registers::*;

    use std::{thread,time};
    use std::time::Duration;
    use crate::memory::memory::MemoryStruct;
    const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);
    
    /*enum FlagR{
        Set,
        Unset,
        Keep, 
        Function(&dyn Fn()->bool) //What should the argument here be?
    }
    enum FlagT{
        ClearAll,
        KeepAll,
        Specific([FlagR;4]) //Nah this is dumb.
    }*/
    enum InterruptState{
        AlmostEnabled,
        Enabled,
        AlmostDisabled,
        Disabled,
    }

    pub struct CpuStruct{
        reg_set: RegStruct,
        memory_ref:&'static MemoryStruct,
        function_lookup:[FunFind;63],
        cb_block_lookup:[FunFind;11],
        current_command:u8
        //Preprocess Args
        //used for mem reads to HL, failed conditional jumps
        //argument:Argument;
    }
    pub struct FunFind{
        mask:u8,
        value:u8,
        function:  fn(&mut CpuStruct, Arg)->bool,//,argument Arg. returns false if we have a longer wait.
        wait:u8,
        wait_cond:Option<u8>,
        //flags: FlagS,
        //bytes: u8//1,2,3, measures the enums.
    }
    impl FunFind{
        fn fun_find(mask: u8, value: u8, function:fn(&mut CpuStruct, Arg)->bool, wait:u8)->Self{
            Self{
                mask,
                value,
                function,
                wait,
                wait_cond:None
            }
        }
        fn fun_find_w(mask: u8, value: u8, fun:fn(&mut CpuStruct, Arg)->bool, wait:u8, wait_cond:u8 )->Self{
            Self{
                mask: mask,
                value:value,
                function:fun,
                wait:wait,
                wait_cond: Some(wait_cond)
            }
        }
    } 
    enum Arg{
        SingleRegArg(registers::SingleReg),
        DoubleRegArg(registers::DoubleReg),
        //PairSingleReg(registers::SingleReg,RegStruct:SingleReg)
        MemReg(registers::DoubleReg),
        StackReg(registers::DoubleReg),
        Cond(bool),
        Imm8(u8),
        Imm16(u16),
        PairSingleReg(registers::SingleReg,registers::SingleReg),
        Empty
    }
    impl CpuStruct{
        fn new() -> Self{
            let mmy: &mut MemoryStruct = &mut MemoryStruct::init_memory();
            Self{
                memory_ref:mmy,
                reg_set:RegStruct::build_registers(mmy),
                current_command:0x00, //initalize to a noop
                function_lookup:[
                    //Block 1,
                    FunFind::fun_find(0xff,0x00,Self::nop,1), //done
                    FunFind::fun_find(0xcf,0x01,Self::str_r16_imm,3),  //done
                    FunFind::fun_find(0xcf,0x02,Self::str_addr_acc,2),
                    FunFind::fun_find(0xcf,0x03,Self::inc_r16,2),//done
                    FunFind::fun_find(0xc7,0x04,Self::inc_r8,1),//done
                    FunFind::fun_find_w(0xc7,0x05,Self::dec_r8,1,2),//done
                    FunFind::fun_find(0xc7,0x06,Self::str_r8_imm,2), //note, both loads and stores are ld, so this is ld_r8_imm8
                    FunFind::fun_find(0xff,0x07,Self::rlca,1),//done
                    FunFind::fun_find(0xff,0x08,Self::ld_imm_sp,5),//done
                    FunFind::fun_find(0xcf,0x09,Self::add_hl,2),//
                    FunFind::fun_find(0xcf,0x0a,Self::ld_acc_addr,5),//done
                    FunFind::fun_find(0xcf,0x0b,Self::dec_r16,2),//done
                    FunFind::fun_find(0xff,0x0f,Self::rrca,1),//done
                    FunFind::fun_find(0xff,0x1f,Self::rra,1),//done
                    FunFind::fun_find(0xff,0x2f,Self::cpl,1),//done
                    FunFind::fun_find(0xff,0x3f,Self::ccf,1),//done
                    FunFind::fun_find(0xff,0x17,Self::rla,1),//done
                    FunFind::fun_find(0xff,0x18,Self::jr_imm,3),//?
                    FunFind::fun_find_w(0xe7,0x20,Self::jr_cond,3,5),//
                    FunFind::fun_find(0xff,0x27,Self::daa,1),//?
                    FunFind::fun_find(0xff,0x37,Self::scf,1),//done
                    FunFind::fun_find(0xff,0x10,Self::stop,0),//?   
                    FunFind::fun_find(0xff,0x76,Self::halt,0),//?
                    FunFind::fun_find_w(0xc0,0x40,Self::ld_r8_r8,1,2),
                    FunFind::fun_find_w(0xf8,0x80,Self::add_r8,1,2),
                    FunFind::fun_find_w(0xf8,0x88,Self::adc_r8,1,2),
                    FunFind::fun_find_w(0xf8,0x90,Self::sub_r8,1,2),
                    FunFind::fun_find_w(0xf8,0x98,Self::subc_r8,1,2),
                    FunFind::fun_find_w(0xf8,0xa0,Self::and_r8,1,2),
                    FunFind::fun_find_w(0xf8,0xa8,Self::xor_r8,1,2),
                    FunFind::fun_find_w(0xf8,0xb0,Self::or_r8,1,2),
                    FunFind::fun_find_w(0xf8,0xb8,Self::cp_r8,1,2),
                    FunFind::fun_find_w(0xff,0xc6,Self::add_imm,1,2),
                    FunFind::fun_find_w(0xff,0xce,Self::adc_imm,1,2),
                    FunFind::fun_find_w(0xff,0xd6,Self::sub_imm,1,2),
                    FunFind::fun_find_w(0xff,0xde,Self::subc_imm,1,2),
                    FunFind::fun_find_w(0xff,0xe6,Self::and_imm,1,2),
                    FunFind::fun_find_w(0xff,0xee,Self::xor_imm,1,2),
                    FunFind::fun_find_w(0xff,0xf6,Self::or_imm,1,2),
                    FunFind::fun_find_w(0xff,0xfe,Self::cp_imm,1,2),
                    FunFind::fun_find_w(0xe7,0xc0,Self::ret_cond,5,2),
                    FunFind::fun_find(0xff,0xc9,Self::ret,4),
                    FunFind::fun_find(0xff,0xd9,Self::reti,4),
                    FunFind::fun_find_w(0xe7,0xc2,Self::jp_cond_imm,3,2),
                    FunFind::fun_find(0xff,0xc3,Self::jp_imm,4),
                    FunFind::fun_find(0xff,0xc9,Self::jp_hl,1),
                    FunFind::fun_find_w(0xe7,0xc4,Self::call_cond,6,3),
                    FunFind::fun_find_w(0xff,0xcd,Self::call_imm,6,3),
                    FunFind::fun_find(0xe7,0xc7,Self::rst,4),
                    FunFind::fun_find(0xcf,0xc1,Self::pop,3),
                    FunFind::fun_find(0xcf,0xc5,Self::push,4),
                    FunFind::fun_find(0xff,0xcb,Self::cb_block,0),
                    FunFind::fun_find(0xff,0xe2,Self::ldh_c,1),
                    FunFind::fun_find(0xff,0xe0,Self::ldh_imm8,2),
                    FunFind::fun_find(0xff,0xeb,Self::ldh_imm16,3),
                    FunFind::fun_find(0xff,0xf2,Self::str_c,1),
                    FunFind::fun_find(0xff,0xf0,Self::str_imm8,2),
                    FunFind::fun_find(0xff,0xfb,Self::str_imm16,3),
                    FunFind::fun_find(0xff,0xe8,Self::add_sp_imm8,4), //
                    FunFind::fun_find(0xff,0xf8,Self::ld_hl_imm8,3),
                    FunFind::fun_find(0xff,0xf8,Self::ld_sp_hl,2),
                    FunFind::fun_find(0xff,0xf3,Self::di,1),
                    FunFind::fun_find(0xff,0xf8,Self::ei,1)
                ],
                cb_block_lookup:[
                    FunFind::fun_find_w(0xf8,0x00,Self::rlc_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x08,Self::rrc_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x10,Self::rl_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x18,Self::rr_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x20,Self::sla_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x28,Self::sra_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x30,Self::swap_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x38,Self::srl_r8,2,4),
                    FunFind::fun_find_w(0xc0,0x40,Self::bit,2,3),
                    FunFind::fun_find_w(0xc0,0x80,Self::res,2,4),
                    FunFind::fun_find_w(0xc0,0x90,Self::set,2,4)
                ]
            }//Find a different way of doing this:
            //Break things apart according to our old pipeline model
        }
        fn get_double_register_from_opcode(){

        }
        fn get_arg_val(arg:Arg){ //Holy mother of polymorphism

        }
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            let current_command:u8 = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1));
            //let first_two:u8 = current_command >> 6
            //static masks:[u8]=[0xFF,0xCF,0xE7,0xC0,0xC7,0xF8]
            let mut taken:bool = false;
            for x in self.function_lookup{
                if current_command & x.mask == x.value{
                    let argument:Arg = match x.mask{
                        0xff => Arg::Empty,
                        0xcf => Arg::DoubleRegArg(self.reg_set.r16_op(current_command)),
                        0xe7 => Arg::Cond(self.reg_set.get_cond(current_command)),
                        0xc0 => Arg::PairSingleReg(self.reg_set.r8_op_mid(current_command),self.reg_set.r8_op_end(current_command)),
                        0xc7 => Arg::SingleRegArg(self.reg_set.r8_op_mid(current_command)),
                        0xf8 => Arg::SingleRegArg(self.reg_set.r8_op_end(current_command)),
                        _ => unreachable!()
                        //let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
                    };
                    let mut result:bool = (x.function)(self,argument);
                    if (x.function)(self,argument){
                        self.wait(x.wait);

                    }else{
                        self.wait(x.wait_cond.unwrap());
                    }; //Evaluate for sanity

                    taken=true;
                    break;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }
            //self.reg_set.increment_pc()

        }
        // A lot of these functions have a few basic cases:
        // Function, Register|DoubleRegister, register2|doubleRegister Immediate, Memory, ImmMemory(either FF00+C or NN), cond
        // Across an ALU with only a few functions
        // Add,Subtract,compare, Inc, Dec, RLA,RRA,  RLCA RRCA 
        // SCF(set carry flag) ccf(change carry flag), which are really just  bit swaps, and bitswaps[xors with masks]
        // Grab bit
        // push/pop, but these are just like a load, and a stack pointer operation
        // Jump is just mapping to the PC
        // and, or, xor, addc, subc, DAA(the 16->digital), swap, which swaps nibbles
        //Accessing (DR) seems to take a logical cycle
        //Accessing (nn) seems to take two, because we have the memory read for the next two instructions, then the memory read for that address
        //call 
        /*enum alu_reg{
            
        }
        enum operand{
            Register(alu_reg),
            DoubleReg(alu_reg,alu_reg),
        }*/

        fn alu_add(&mut self,arg1:Arg,arg2:Arg,arg3:Arg){ //So we have like, 2 inputs, and and output 

        }
        fn alu_add(&mut self, arg:Arg) ->

        } 



        fn nop(&mut self, arg:Arg)->bool{
            true
        }

        fn ld_imm_sp(&mut self,arg:Arg)->bool{
            self.reg_set.set_double_register(registers::DoubleReg::SP,self.memory_ref.grab_memory_16(self.reg_set.increment_pc(1)));
            self.reg_set.increment_pc(1);
            true
        }
        fn rrc_r8(&mut self, arg:Arg)->bool{
            let arg_reg = match arg{
                Arg::SingleRegArg(reg) => reg,
                _ => unreachable!()
            };
            if self.reg_set.get_bit(arg_reg, 0){
                self.reg_set.set_flag(Flag::Carry); //Rotate right
            } //Rotate right
            self.reg_set.change_single_register(arg_reg, &|x| x.rotate_right(1));
            true
        }
        fn rrca(&mut self,arg:Arg)->bool{
            self.rrc_r8(Arg::SingleRegArg(registers::SingleReg::A))
        }
        fn rlc_r8(&mut self, arg:Arg) -> bool{
            let arg_reg = match arg{
                Arg::SingleRegArg(reg) => reg,
                _ => unreachable!()
            };
            if self.reg_set.get_bit(arg_reg, 7){
                self.reg_set.set_flag(Flag::Carry,); //Rotate right
            }
            self.reg_set.change_single_register(arg_reg, &|x| x.rotate_left(1));
            true
        }
        fn rlca(&mut self, arg:Arg)->bool{
            self.rlc_r8(Arg::SingleRegArg(registers::SingleReg::A));
            return true
        }
        fn rla(&mut self, arg:Arg)->bool{ //Rotate register a to the left _through_ the carry bti .
            let carry:bool = self.reg_set.get_flag(Flag::Carry);
            let top:bool = self.reg_set.get_bit(SingleReg::A,7);
            if top{
                self.reg_set.set_flag(Flag::Carry);
            }
            self.reg_set.change_single_register(registers::SingleReg::A, &|x| x<<1 + (carry as u8)); //Check to see if I need wrapping left shi
            true
        }
        fn rra(&mut self,arg:Arg)->bool{ //Need to rewrite.
            let carry:bool = self.reg_set.get_flag(Flag::Carry);
            let bottom:bool = self.reg_set.get_bit(registers::SingleReg::A, 0);
            if bottom{
                self.reg_set.set_flag(Flag::Carry);
            }
            self.reg_set.change_single_register(registers::SingleReg::A, &|x| (x>>1) + ((carry as u8)<<7));
            true
        }
        fn jr_imm(&mut self, arg:Arg)->bool{
            let next_value:i8 = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1)) as i8;
            self.reg_set.change_double_register(registers::DoubleReg::PC, &|x| x.wrapping_add_signed(next_value.into()));
            true
        }
        fn jr_cond(&mut self, arg:Arg)->bool{
            if !self.reg_set.get_cond(self.current_command){
                false
            }
            else{
                self.jr_imm(arg)
            }
        }
        fn daa(&mut self, arg:Arg)->bool{
            let subtract = self.reg_set.get_flag(registers::Flag::Neg);
            let hcarry = self.reg_set.get_flag(registers::Flag::HalfCarry);
            let carry = self.reg_set.get_flag(registers::Flag::Carry);
            
            //To complete
            true
        }

        fn str_r16_imm(&mut self, arg:Arg)->bool{ //Properly LD r16 imm16
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.set_double_register(reg_pair,self.memory_ref.grab_memory_16(self.reg_set.get_double_register(reg_pair)));
            //This may actually also be like... just run str r8 imm twice.
            true
        }
        fn str_addr_acc(&mut self, arg:Arg)->bool{
            let reg_pair:DoubleReg = self.reg_set.r16_mem(self.current_command);
            self.memory_ref.set_memory_8(self.reg_set.get_double_register(reg_pair),self.reg_set.get_acc());
            true
        }
        fn inc_r8(&mut self, arg:Arg)->bool{
            let reg:SingleReg = match arg{
                Arg::SingleRegArg(rg)=>rg,
                _ => unreachable!()
            }; //self.reg_set.r8_op_mid(self.current_command);
            self.reg_set.change_single_register(reg, &|x| x+1);
            !matches!(reg,SingleReg::Memptr)
        }
        fn dec_r8(&mut self, arg:Arg)->bool{
            let reg:SingleReg = self.reg_set.r8_op_mid(self.current_command);
            self.reg_set.change_single_register(reg, &|x| x-1);
            !matches!(reg,SingleReg::Memptr)
        }
        fn inc_r16(&mut self, arg:Arg)->bool{
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.change_double_register(reg_pair,&|x| x+1);
            true
        }
        fn dec_r16(&mut self, arg:Arg)->bool{
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.change_double_register(reg_pair,&|x| x-1);
            true
        }
        fn str_r8_imm(&mut self, arg:Arg)->bool{
            let reg:SingleReg = self.reg_set.r8_op_mid(self.current_command);
            let imm:u8 = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1));
            self.reg_set.set_single_register(reg,imm);
            true
        }
        fn ld_r8_r8(&mut self, arg:Arg)->bool{
            let (reg_dest, reg_src) = match arg{
                Arg::PairSingleReg( reg1,reg2) => (reg1,reg2),
                _ => unreachable!()
            };
            self.reg_set.set_single_register(reg_dest,self.reg_set.get_register(reg_src));
        
            return !matches!(reg_src,SingleReg::Memptr) && !matches!(reg_dest,SingleReg::Memptr)
        }
        fn ld_acc_addr(&mut self, arg:Arg)->bool{ //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg:DoubleReg = self.reg_set.r16_mem(self.current_command);
            self.reg_set.set_acc(self.memory_ref.grab_memory_8(self.reg_set.get_double_register(reg)));
            if matches!(reg,DoubleReg::HLP) || matches!(reg,DoubleReg::HLM) {
                self.reg_set.set_double_register(reg,0);
            }
            true
        }
        fn add_hl(&mut self, arg:Arg)->bool{
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            let double_reg_val:u16 = self.reg_set.get_double_register(reg_pair);
            self.reg_set.change_double_register(DoubleReg::HL, &|x|x+double_reg_val);
            //TODO: Flags
            true
        }

        fn cpl(&mut self, arg:Arg)->bool{ //Invert A
            self.reg_set.change_single_register(SingleReg::A,&|x| !x);
            self.reg_set.set_flag(Flag::HalfCarry);
            self.reg_set.set_flag(Flag::Neg);
            true
        }
        fn ccf(&mut self, arg:Arg)->bool{
            self.reg_set.flip_carry();
            true
        }
        fn scf(&mut self, arg:Arg)->bool{
            self.reg_set.set_flag(Flag::Carry);
            true
        }
        fn add_r8(&mut self, arg:Arg)->bool{ 
            self.reg_set.apply_fun_to_acc(arg, &|x|x+self.reg_set.get_register(process_single_arg(arg)))
        } 
        fn adc_r8(&mut self, arg:Arg)->bool{
            let arg_reg = process_single_arg();
            let carry = self.reg_set.get_flag(Flag::Carry);
            self.reg_set.apply_fun_to_acc(arg_reg, &|x|x+carry+self.reg_set.get_register(arg_reg))
        } 
        fn sub_r8(&mut self, arg:Arg)->bool{
            self.reg_set.apply_fun_to_acc(arg_reg, &|x|x-self.reg_set.get_register(process_single_arg(arg)))
        }
        fn subc_r8(&mut self, arg:Arg)->bool{
            let arg_reg = process_single_arg();
            let carry = self.reg_set.get_flag(Flag::Carry);
            self.reg_set.apply_fun_to_acc(arg_reg, &|x| x+carry+self.reg_set.get_register(arg_reg))
        } 
        fn and_r8(&mut self, arg:Arg)->bool{
            self.reg_set.apply_fun_to_acc(arg_reg, &|x| x&self.reg_set.get_register(process_single_arg(arg)))
        } 
        fn xor_r8(&mut self, arg:Arg)->bool{
            self.reg_set.apply_fun_to_acc(arg_reg, &|x| x^self.reg_set.get_register(process_single_arg(arg)))
        } 
        fn or_r8(&mut self, arg:Arg)->bool{
            self.reg_set.apply_fun_to_acc(arg_reg, &|x| x|self.reg_set.get_register(process_single_arg(arg)))
        }
        fn cp_r8(&mut self, arg:Arg)->bool{
            let arg_reg = get_arg_val_single(arg);
            let acc = self.reg_set.get_register(SingleReg::A);
/*
Figure out half carry
*/
            self.reg_set.set_flags((arg_reg-acc == 0),true,true,(arg_reg>a))
            //self.reg_set.apply_fun_to_acc(arg_reg, &|x|x&self.reg_set.get_register(process_single_arg(arg)))
        } 
        fn add_imm(&mut self, arg:Arg)->bool{
        }
        fn adc_imm(&mut self, arg:Arg)->bool{

        }
        fn sub_imm(&mut self, arg:Arg)->bool{
        }
        fn subc_imm(&mut self, arg:Arg)->bool{
        }
        fn and_imm(&mut self, arg:Arg)->bool{
        }
        fn xor_imm(&mut self, arg:Arg)->bool{
        }
        fn or_imm(&mut self, arg:Arg)->bool{
        }
        fn cp_imm(&mut self, arg:Arg)->bool{
        }
        fn ret_cond(&mut self, arg:Arg)->bool{
        } 
        fn ret(&mut self, arg:Arg)->bool{
        }
        fn reti(&mut self, arg:Arg)->bool{
        } 
        fn jp_cond_imm(&mut self, arg:Arg)->bool{
        } 
        fn jp_imm(&mut self, arg:Arg)->bool{
        } 
        fn jp_hl(&mut self, arg:Arg)->bool{
        } 
        fn call_cond(&mut self, arg:Arg)->bool{
        } 
        fn call(&mut self, arg:Arg)->bool{
            self.reg_set.get_double_register()
        } 
        fn rst(&mut self, arg:Arg)->bool{
        }
        fn pop(&mut self, arg:Arg)->bool{
        }
        fn push(&mut self, arg:Arg)->bool{
        }
        fn ldh_c(&mut self, arg:Arg)->bool{ //A = mem($FF00 + c)
        } 
        fn ldh_imm8(&mut self, arg:Arg)->bool{
        }
        fn ldh_imm16(&mut self, arg:Arg)->bool{
        }
        fn str_c(&mut self, arg:Arg)->bool{ //Store A at address $FF00+C 
        } 
        fn str_imm8(&mut self, arg:Arg)->bool{
        } 
        fn str_imm16(&mut self, arg:Arg)->bool{
        }
        fn add_sp_imm8(&mut self, arg:Arg)->bool{
        //Remember, this is a signed value!
        }
        fn ld_hl_imm8(&mut self, arg:Arg)->bool{
        }
        fn ld_sp_hl(&mut self, arg:Arg)->bool{
        }
        fn di(&mut self, arg:Arg)->bool{
        }
        fn ei(&mut self, arg:Arg)->bool{
        } 
        fn rl_r8(&mut self, arg:Arg)->bool{
        }
        fn rr_r8(&mut self, arg:Arg)->bool{
        } 
        fn sla_r8(&mut self, arg:Arg)->bool{
        } 
        fn sra_r8(&mut self, arg:Arg)->bool{
        } 
        fn swap_r8(&mut self, arg:Arg)->bool{
        } 
        fn srl_r8(&mut self, arg:Arg)->bool{
        } 
        fn bit(&mut self, arg:Arg)->bool{
        } 
        fn res(&mut self, arg:Arg)->bool{
        } 
        fn set(&mut self, arg:Arg)->bool{
        }
        fn stop(&mut self, arg:Arg)->bool{
            loop {
                self.wait(1);
                //PAUSE THE GPU
                //BREAK IF BUTTON PRESSED.
            }
        }
        fn halt(&mut self, arg:Arg)->bool{
            loop { 
                CpuStruct::wait(1) //Enter low power mode until an interrupt
            }
        }
        fn cb_block(&mut self, arg:Arg )->bool{
            let current_command:u8 = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1));
            let mut taken = false;
            for x in self.cb_block_lookup{
                if current_command & x.mask == x.value{
                    (x.function)(self,arg); //Evaluate for sanity
                    taken=true;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }
            true

            //grab the next piece of memory, but we use the CB table.
        }
        fn wait(&mut self,cycles:u8){
            //4.19 mhz * 4 t cycles 
            thread::sleep(4*CLOCK_PERIOD*cycles.into());
        }
    }
}