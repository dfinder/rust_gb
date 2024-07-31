

pub mod cpu { 
    use crate::registers::registers::{self, SingleReg};
    use crate::registers::registers::*;
    use crate::cpu_state::cpu_state::*;
    use std::{thread,time};
    use std::time::Duration;
    use crate::memory::memory::MemoryStruct;
    const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);
    type CPUFunct =  fn(&mut CpuStruct);
    enum InterruptState{
        Enabled, //Despite naming, it's really that we have E, DI, AD as "enmabled" states
        DisableInterrupt,
        AlmostDisabled,
        EnableInterrupt, 
        AlmostEnabled,
        Disabled,
    }

    pub struct CpuStruct{
        cpu_state: CpuState,
        function_lookup:[FunFind;63],
        cb_block_lookup:[FunFind;11],
        current_command:u8,
        extra_waiting:bool,
        interrupt:InterruptState,
        is_cb:bool,
        //Preprocess Option<Operand>
        //used for mem reads to HL, failed conditional jumps
        //argument:Argument;
    }
    pub struct FunFind{
        mask:u8,
        value:u8,
        function: CPUFunct ,//,argument Arg. returns false if we have a longer wait.
        wait:u8,
        wait_cond:Option<u8>,
        //flags: FlagS,
        //bytes: u8//1,2,3, measures the enums.
    }
    impl FunFind{
        fn fun_find(mask: u8, value: u8, function:CPUFunct, wait:u8)->Self{
            Self{
                mask,
                value,
                function,
                wait,
                wait_cond:None,
            }
        }
        fn fun_find_w(mask: u8, value: u8, function:CPUFunct, wait:u8, wait_cond:u8 )->Self{
            Self{
                mask: mask,
                value:value,
                function:function,
                wait:wait,
                wait_cond: Some(wait_cond),
            }
        }
    }
    impl CpuStruct{
        pub fn new() -> Self{
            Self{
                cpu_state: CpuState::new(),
                current_command:0x00, //initalize to a noop
                function_lookup:[
                    //Block 1,
                    FunFind::fun_find(0xff,0x00,CpuStruct::nop,1), //done
                    FunFind::fun_find(0xcf,0x01,CpuStruct::ldi_r16,3),  //done
                    FunFind::fun_find(0xcf,0x02,CpuStruct::str_acc_rmem,2),
                    FunFind::fun_find(0xcf,0x03,CpuStruct::inc_r16,2),//done
                    FunFind::fun_find(0xc7,0x04,CpuStruct::inc_r8,1),//done
                    FunFind::fun_find(0xc7,0x05,CpuStruct::dec_r8,1),//done
                    FunFind::fun_find(0xc7,0x06,CpuStruct::ldi_r8,2), //done
                    FunFind::fun_find(0xff,0x07,CpuStruct::rlc,1),//done
                    FunFind::fun_find(0xff,0x08,CpuStruct::ld_imm_sp,5),//done
                    FunFind::fun_find(0xcf,0x09,CpuStruct::add_hl,2),//
                    FunFind::fun_find(0xcf,0x0a,CpuStruct::ld_acc_addr,5),//done
                    FunFind::fun_find(0xcf,0x0b,CpuStruct::dec_r16,2),//done
                    FunFind::fun_find(0xff,0x0f,CpuStruct::rrc,1),//done
                    FunFind::fun_find(0xff,0x1f,CpuStruct::rr,1),//done
                    FunFind::fun_find(0xff,0x2f,CpuStruct::cpl,1),//done
                    FunFind::fun_find(0xff,0x3f,CpuStruct::ccf,1),//done
                    FunFind::fun_find(0xff,0x17,CpuStruct::rl,1),//done
                    FunFind::fun_find(0xff,0x18,CpuStruct::jr_imm,3),//?
                    FunFind::fun_find_w(0xe7,0x20,CpuStruct::jr_cond,3,5),//
                    FunFind::fun_find(0xff,0x27,CpuStruct::daa,1),//?
                    FunFind::fun_find(0xff,0x37,CpuStruct::scf,1),//done
                    FunFind::fun_find(0xff,0x10,CpuStruct::stop,0),//?   
                    FunFind::fun_find(0xff,0x76,CpuStruct::halt,0),//?
                    FunFind::fun_find_w(0xc0,0x40,CpuStruct::ld_r8_r8,1,2),
                    FunFind::fun_find_w(0xf8,0x80,CpuStruct::add,1,2),
                    FunFind::fun_find_w(0xf8,0x88,CpuStruct::adc,1,2),
                    FunFind::fun_find_w(0xf8,0x90,CpuStruct::sub,1,2),
                    FunFind::fun_find_w(0xf8,0x98,CpuStruct::subc,1,2),
                    FunFind::fun_find_w(0xf8,0xa0,CpuStruct::and,1,2),
                    FunFind::fun_find_w(0xf8,0xa8,CpuStruct::xor,1,2),
                    FunFind::fun_find_w(0xf8,0xb0,CpuStruct::or,1,2),
                    FunFind::fun_find_w(0xf8,0xb8,CpuStruct::cp,1,2),
                    FunFind::fun_find_w(0xff,0xc6,CpuStruct::add,1,2),
                    FunFind::fun_find_w(0xff,0xce,CpuStruct::adc,1,2),
                    FunFind::fun_find_w(0xff,0xd6,CpuStruct::sub,1,2),
                    FunFind::fun_find_w(0xff,0xde,CpuStruct::subc,1,2),
                    FunFind::fun_find_w(0xff,0xe6,CpuStruct::and,1,2),
                    FunFind::fun_find_w(0xff,0xee,CpuStruct::xor,1,2),
                    FunFind::fun_find_w(0xff,0xf6,CpuStruct::or,1,2),
                    FunFind::fun_find_w(0xff,0xfe,CpuStruct::cp,1,2),
                    FunFind::fun_find_w(0xe7,0xc0,CpuStruct::ret_cond,5,2),
                    FunFind::fun_find(0xff,0xc9,CpuStruct::ret,4),
                    FunFind::fun_find(0xff,0xd9,CpuStruct::reti,4),
                    FunFind::fun_find_w(0xe7,0xc2,CpuStruct::jp_cond_imm,3,2),
                    FunFind::fun_find(0xff,0xc3,CpuStruct::jp_imm,4),
                    FunFind::fun_find(0xff,0xc9,CpuStruct::jp_hl,1),
                    FunFind::fun_find_w(0xe7,0xc4,CpuStruct::call_cond,6,3),
                    FunFind::fun_find_w(0xff,0xcd,CpuStruct::call_imm,6,3),
                    FunFind::fun_find(0xe7,0xc7,CpuStruct::rst,4),
                    FunFind::fun_find(0xcf,0xc1,CpuStruct::pop,3),
                    FunFind::fun_find(0xcf,0xc5,CpuStruct::push,4),
                    FunFind::fun_find(0xff,0xcb,CpuStruct::cb_block,1),
                    FunFind::fun_find(0xff,0xe2,CpuStruct::str_c,1), //We're storing if we're mapping to memory, we're loadingif we'r 
                    FunFind::fun_find(0xff,0xe0,CpuStruct::str_imm8,2),
                    FunFind::fun_find(0xff,0xea,CpuStruct::str_imm16,3),

                    FunFind::fun_find(0xff,0xf0,CpuStruct::ld_imm8,2),
                    FunFind::fun_find(0xff,0xf2,CpuStruct::ld_c,1),
                    FunFind::fun_find(0xff,0xfa,CpuStruct::ld_imm16,3),
                    FunFind::fun_find(0xff,0xe8,CpuStruct::add_sp_imm8,4), //
                    FunFind::fun_find(0xff,0xf8,CpuStruct::ld_hl_imm8,3),
                    FunFind::fun_find(0xff,0xf9,CpuStruct::ld_sp_hl,2),
                    FunFind::fun_find(0xff,0xf3,CpuStruct::di,1),
                    FunFind::fun_find(0xff,0xfb,CpuStruct::ei,1)
                ],
                cb_block_lookup:[
                    FunFind::fun_find_w(0xf8,0x00,CpuStruct::rlc,2,4),
                    FunFind::fun_find_w(0xf8,0x08,CpuStruct::rrc,2,4),
                    FunFind::fun_find_w(0xf8,0x10,CpuStruct::rl,2,4),
                    FunFind::fun_find_w(0xf8,0x18,CpuStruct::rr,2,4),
                    FunFind::fun_find_w(0xf8,0x20,CpuStruct::sla,2,4),
                    FunFind::fun_find_w(0xf8,0x28,CpuStruct::sra,2,4),
                    FunFind::fun_find_w(0xf8,0x30,CpuStruct::swap,2,4),
                    FunFind::fun_find_w(0xf8,0x38,CpuStruct::srl,2,4),
                    FunFind::fun_find_w(0xc0,0x40,CpuStruct::bit,2,3),
                    FunFind::fun_find_w(0xc0,0x80,CpuStruct::res,2,4),
                    FunFind::fun_find_w(0xc0,0x90,CpuStruct::set,2,4)
                ],
                extra_waiting:false,
                interrupt:InterruptState::Enabled,
                is_cb: false
            }//Find a different way of doing this:
            //Break things apart according to our old pipeline model
        }
        fn btt (&mut self,b: bool)->i8{
            -1 + (2*(b as i8))
        }
        fn is_imm(&mut self)->bool{
            self.current_command & 0x11000000 > 0 
        }
        /*fn get_6_bit_arg(&mut self,ff:FunFind)->Option<Operand>{
            if ff.value == 0x40 && ff.wait==1{
                return Some(Operand::PairSingleReg(self.cpu_state.r8_op_mid(ff.value), self.cpu_state.r8_op_end(ff.value)))
            }
            Some(Operand::BitsSingleReg((ff.value & 127) >> 4, self.cpu_state.r8_op_end(ff.value)))

        }*/
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            let current_pc = self.cpu_state.inc_pc();
            self.current_command = self.cpu_state.get_byte(current_pc);
            //let first_two:u8 = current_command >> 6
            //static masks:[u8]=[0xFF,0xCF,0xE7,0xC0,0xC7,0xF8]
            let mut taken:bool = false;
            self.extra_waiting = false;
            self.interrupt = match self.interrupt{
                InterruptState::Enabled|InterruptState::AlmostEnabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => InterruptState::AlmostEnabled,
                InterruptState::DisableInterrupt => InterruptState::AlmostDisabled,
                InterruptState::Disabled|InterruptState::AlmostDisabled => InterruptState::Disabled, 
            };
            match self.interrupt{
                InterruptState::Enabled|InterruptState::DisableInterrupt|InterruptState::AlmostDisabled => todo!(),
                _ => ()

            }
            for fun_entry in &self.function_lookup[..]{
                if (self.current_command & fun_entry.mask) == fun_entry.value{
                    (fun_entry.function)(self);
                    if self.extra_waiting{
                        CpuStruct::wait(fun_entry.wait);
                    }else{
                        CpuStruct::wait(fun_entry.wait_cond.unwrap());
                    }; //Evaluate for sanity

                    taken=true;
                    break;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }
            //Manage interrupts
        }
        fn cond(&mut self)->bool{
            self.cpu_state.get_cond(self.current_command)
        }
        fn alu_register(&mut self)->SingleReg{
            if self.current_command >0x11000000 { 
                SingleReg::A
             }else{
                 self.cpu_state.get_r8_end(self.current_command)
             }
        }
        fn alu_operand(&mut self)->u8{
            if self.current_command >0x11000000 { 
               self.cpu_state.get_imm8()
            }else{
                self.extra_waiting = true;
                let register= self.cpu_state.get_r8_end(self.current_command);
                self.cpu_state.get_r8_val(register)
            }
        }
        fn nop(&mut self){
            ()
        }
        // Rotates
        fn rl(&mut self){
            let val = self.alu_operand();
            let carry:bool = self.cpu_state.get_flag(Flag::Carry);
            let reg = self.alu_register();
            self.cpu_state.change_r8(reg, &|x| x<<1 + (carry as u8)); //Check to see if I need wrapping left shi
            self.cpu_state.set_flag(Flag::Carry,val>127);
        }
        fn rlc(&mut self){ //Rotate left
            let val = self.alu_operand();
            let reg = self.alu_register();
            self.cpu_state.set_flag(Flag::Carry,val>127); 
            self.cpu_state.change_r8(reg, &|x| x.rotate_left(1));
        }
        fn rr(&mut self){
            let reg = self.alu_register();
            let val = self.alu_operand();
            //reg=self.
            let carry:bool = self.cpu_state.get_flag(Flag::Carry);
            let bottom:bool = (val % 2)==1;
            self.cpu_state.set_flag(Flag::Carry,bottom);
            self.cpu_state.change_r8(reg, &|x| (x>>1) + ((carry as u8)<<7));
        } 
        fn rrc(&mut self){

            let value = self.alu_operand();
            let reg = self.alu_register();
            self.cpu_state.set_flag(Flag::Carry,(value % 2)==1); 
            self.cpu_state.change_r8(reg, &|x| x.rotate_right(1));
        }

        fn daa(&mut self){
            let subtract = self.cpu_state.get_flag(registers::Flag::Neg);
            let hcarry = self.cpu_state.get_flag(registers::Flag::HalfCarry);
            let carry = self.cpu_state.get_flag(registers::Flag::Carry);
            let mut offset:u8= 0;
            let a_val = self.cpu_state.get_acc();
            if (!subtract && a_val&0xf > 0x9) || hcarry{
                offset |= 0x06
            }
            if (!subtract && a_val > 0x99) || carry{
                offset |= 0x06
            }
            //let fun = &|x|x+offset;
            if subtract{
                self.cpu_state.apply_fun_to_acc(&|x|x.wrapping_sub(offset));
            }else{
                self.cpu_state.apply_fun_to_acc(&|x|x.wrapping_add(offset));
            }
            self.cpu_state.set_flag(Flag::HalfCarry,false);
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==0);
            self.cpu_state.set_flag(Flag::Zero, acc>0x99);
        }
        //Load Immediate
        fn ldi_r16(&mut self){ //0x01
            let reg_pair: DoubleReg = self.cpu_state.r16_tbl(self.current_command);
            let imm2:u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(reg_pair,imm2);
            //This may actually also be like... just run str r8 imm twice.
        }
        fn ldi_r8(&mut self){
            let arg_reg: SingleReg = self.cpu_state.get_r8_mid(self.current_command);
            let imm:u8 = self.cpu_state.get_imm8();
            self.cpu_state.set_r8(arg_reg,imm);
        }
        //Stores
        fn str_acc_rmem(&mut self){//0x02
            let arg_reg: DoubleReg = self.cpu_state.r16_mem_tbl(self.current_command);
            let mem_addr: u16 = self.cpu_state.get_r16_val(arg_reg);
            let acc_val = self.cpu_state.get_acc();
            self.cpu_state.set_byte(mem_addr,acc_val);
        }
        fn str_c(&mut self){ //Store A at address $FF00+C , e2
            let value:u16 = self.cpu_state.get_r8_val(SingleReg::C) as u16 + 0xFF00; 
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_byte(value, acc );
        } 
        fn str_imm8(&mut self){ //e0 
            let reg:u8 = self.cpu_state.get_acc();
            let imm:u16 = self.cpu_state.get_imm8() as u16;
            self.cpu_state.set_byte(imm+0xFF00, reg);
        }
        fn str_imm16(&mut self){ //EA
            let reg:u8 = self.cpu_state.get_acc();
            let imm:u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_byte(imm, reg);
        }
        //Loads 
        fn ld_imm8(&mut self){ //F0
            let imm:u16 = (self.cpu_state.get_imm8() as u16)+0xFF00;
            let mem:u8 = self.cpu_state.get_byte(imm);
            self.cpu_state.set_acc(mem)

        } 
        fn ld_imm16(&mut self){
            let imm:u16 = self.cpu_state.get_imm16();
            let mem:u8 = self.cpu_state.get_byte(imm);
            self.cpu_state.set_acc(mem)
        }
        fn ld_imm_sp(&mut self){
            let imm:u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(registers::DoubleReg::SP,imm);
        }
        fn ld_r8_r8(&mut self){
            let reg_dest = self.cpu_state.get_r8_mid(self.current_command);
            let reg_src = self.cpu_state.get_r8_end(self.current_command);
            let reg_val = self.cpu_state.get_r8_val(reg_src);
            self.cpu_state.set_r8(reg_dest,reg_val);
            self.extra_waiting = matches!(reg_src,SingleReg::Memptr) || !matches!(reg_dest,SingleReg::Memptr)
        }
        fn ld_acc_addr(&mut self){ //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg = self.cpu_state.r16_tbl(self.current_command);
            let mem: u8 = self.cpu_state.get_r16_memory(reg);
            self.cpu_state.set_acc(mem);
            if matches!(reg,DoubleReg::HLP) || matches!(reg,DoubleReg::HLM) {
                self.cpu_state.set_r16_val(reg,0);
            }
        }
        fn ld_c(&mut self){ //A = mem($FF00 + c)
            let addr:u16 = self.cpu_state.get_r8_val(SingleReg::C) as u16 + 0xFF00; 
            let value:u8 = self.cpu_state.get_byte(addr);
            self.cpu_state.set_acc(value);
        } 
        fn ld_hl_imm8(&mut self){ //f8
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let imm = self.cpu_state.get_imm8() as u16;
            self.cpu_state.set_r16_val(DoubleReg::HL, stack_pointer+imm);
        }
        fn ld_sp_hl(&mut self){ //f9
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            self.cpu_state.set_r16_val(DoubleReg::HL,stack_pointer);
        }
        fn inc_r8(&mut self){
            let reg:SingleReg = self.cpu_state.get_r8_mid(self.current_command); //self.cpu_state.r8_op_mid(self.current_command);
            let val:u8 = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, val==0xff);
            self.cpu_state.set_flag(Flag::HalfCarry, val==0x0f);
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.change_r8(reg, &|x| x.wrapping_add(1));
            
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        fn dec_r8(&mut self){
            let reg:SingleReg = self.cpu_state.get_r8_mid(self.current_command);
            let val:u8 = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, val==0x01);
            self.cpu_state.set_flag(Flag::HalfCarry, val==0x10);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.change_r8(reg, &|x| x.wrapping_sub(1));
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        fn inc_r16(&mut self){ //Doesn't affect flags
            let reg_pair:DoubleReg = self.cpu_state.r16_tbl(self.current_command);
            self.cpu_state.change_r16(reg_pair,&|x| x+1);
        }
        fn dec_r16(&mut self){ //doesn't affect flags.
            let reg_pair:DoubleReg = self.cpu_state.r16_tbl(self.current_command);
            self.cpu_state.change_r16(reg_pair,&|x| x-1);
        }
        fn add_hl(&mut self){
            let reg_pair:DoubleReg = self.cpu_state.r16_tbl(self.current_command);
            let operand:u16 = self.cpu_state.get_r16_val(reg_pair);
            let hl_val:u16 = self.cpu_state.get_r16_val(DoubleReg::HL);
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.set_flag(Flag::HalfCarry, ((hl_val&0x0FFF)+operand & 0x0fff)>0x1000);
            self.cpu_state.set_flag(Flag::Carry, None == hl_val.checked_add(operand));
            self.cpu_state.set_flag(Flag::Zero,hl_val.wrapping_add(operand)==0);
            self.cpu_state.change_r16(DoubleReg::HL, &|x|x.wrapping_add(operand));
        }
        fn cpl(&mut self){ //Invert A
            self.cpu_state.change_r8(SingleReg::A,&|x| !x);
            self.cpu_state.set_flag(Flag::HalfCarry,true);
            self.cpu_state.set_flag(Flag::Neg,true);
        }
        fn ccf(&mut self){
            self.cpu_state.flip_carry();
        }
        fn scf(&mut self){
            self.cpu_state.set_flag(Flag::Carry,true);
        }
        fn add(&mut self){ 
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc.wrapping_add(operand)==0);
            self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand) );
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F)) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.apply_fun_to_acc( &|x|x.wrapping_add(operand));
        } 
        fn adc(&mut self){
            let carry = self.cpu_state.get_flag(Flag::Carry);
            let operand = self.alu_operand()+(carry as u8);
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc.wrapping_add(operand)==0);
            self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand));
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F) as u8) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.apply_fun_to_acc( &|x|x.wrapping_add(operand));
        } 
        fn sub(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==operand);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F)<(operand & 0x0F)));
            self.cpu_state.set_flag(Flag::Carry, operand>acc);
            self.cpu_state.apply_fun_to_acc( &|x|x.wrapping_sub(operand))
        }
        fn subc(&mut self){
            let carry:u8 = self.cpu_state.get_flag(Flag::Carry) as u8;
            let operand = self.alu_operand()+carry;
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==operand);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F)<((operand) & 0x0F)));
            self.cpu_state.set_flag(Flag::Carry, operand>acc);
            self.cpu_state.apply_fun_to_acc(&|x|x.wrapping_sub(operand));
        } 
        fn and(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flags(operand&acc == 0, false, true, false);
            self.cpu_state.apply_fun_to_acc(&|x|x&operand);
        } 
        fn xor(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flags(operand^acc == 0 , false, true, false);
            self.cpu_state.apply_fun_to_acc( &|x|x^operand);
        } 
        fn or(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flags(operand|acc == 0 , false, true, false);
            self.cpu_state.apply_fun_to_acc( &|x| x|operand); //what if I were to go even cooler
        }
        fn cp(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==operand);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F)<(operand & 0x0F)));
            self.cpu_state.set_flag(Flag::Carry, operand>acc);
        } 

        fn ret(&mut self){   
            let instruction = self.cpu_state.get_r16_memory_word(DoubleReg::SP);
            self.cpu_state.set_r16_val(DoubleReg::PC, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x+2);
        }
        fn ret_cond(&mut self){
            if self.cond(){
                self.ret();
                self.extra_waiting=true;
            }
        } 
        fn reti(&mut self){
            self.ret();
            self.ei();
        } 
        //Jumps
        fn jr_imm(&mut self){ //Jump Relative
            let imm = self.cpu_state.get_imm8() as u16;
            let pc = self.cpu_state.get_r16_val(registers::DoubleReg::PC);
            self.cpu_state.set_pc(pc+imm);
        }
        fn jr_cond(&mut self){
            if self.cond(){
                self.jr_imm();
                self.extra_waiting=true;
            } 
        }
        fn jp_cond_imm(&mut self){
            if self.cond(){
                self.jp_imm();
                self.extra_waiting=true;
            } 
        }  
        //Jump to 
        fn jp_imm(&mut self){
            let imm = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(registers::DoubleReg::PC, imm);
        } 
        fn jp_hl(&mut self){
            let hl =  self.cpu_state.get_r16_val(registers::DoubleReg::HL);
            self.cpu_state.set_r16_val(registers::DoubleReg::PC,hl)
        } 
        fn call_cond(&mut self){
            if self.cond(){
                self.call_imm();
                self.extra_waiting=true;
            } 
        } 
        fn call_imm(&mut self){
            let addr = self.cpu_state.get_imm16();
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let pc = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_half_word(stack_pointer,pc);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
            self.cpu_state.set_pc(addr);
        } 
        fn rst(&mut self){
            let pc: u16 = self.cpu_state.get_r16_val(DoubleReg::PC);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            self.cpu_state.set_half_word(stack_pointer,pc);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
            self.cpu_state.set_pc((self.current_command & 0x00111000) as u16);
        }
        fn pop(&mut self){
            let operand = self.cpu_state.r16_stk_tbl(self.current_command);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let instruction = self.cpu_state.get_half_word(stack_pointer);
            self.cpu_state.set_r16_val(operand, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x+2);
        }
        fn push(&mut self){
            let operand = self.cpu_state.r16_stk_tbl(self.current_command);
            let value: u16 = self.cpu_state.get_r16_val(operand);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);

            self.cpu_state.set_half_word(stack_pointer, value);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
        }
        fn add_sp_imm8(&mut self){
            let operand:i8 = self.cpu_state.get_imm8() as i8;
            //self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand) );
            //self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F)) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.set_flag(Flag::Zero,false);

            self.cpu_state.change_r16(registers::DoubleReg::SP, &|x| x.wrapping_add_signed(operand as i16));
        }

        fn di(&mut self){
            self.interrupt =match self.interrupt{
                InterruptState::AlmostDisabled|InterruptState::Disabled => InterruptState::Disabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::AlmostDisabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        fn ei(&mut self){
            self.interrupt =match self.interrupt{
                InterruptState::AlmostDisabled|InterruptState::Disabled => InterruptState::AlmostEnabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        } 

        fn sla(&mut self){
            let reg = self.cpu_state.get_r8_end(self.current_command);
            let operand = self.cpu_state.get_r8_val(reg);                
            self.cpu_state.set_flags(operand == 128 || operand==0,false,false,operand>127);
            self.cpu_state.change_r8(reg, &|x|x<<1);
        } 
        fn sra(&mut self){
            let reg = self.cpu_state.get_r8_end(self.current_command);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand == 1 || operand==0,false,false,operand%2 == 1);
            self.cpu_state.change_r8(reg, &|x|(x>>1)+(128 * ((x>127) as u8))); //Sneaky little arithmetic right shift.
        } 
        fn srl(&mut self){
            let reg = self.cpu_state.get_r8_end(self.current_command);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand == 1 || operand==0,false,false,operand%2 == 1);
            self.cpu_state.change_r8(reg, &|x|(x>>1)); 
        } 
        fn swap(&mut self){
            let reg = self.cpu_state.get_r8_end(self.current_command);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand==0,false,false,false);
            self.cpu_state.change_r8(reg, &|x:u8|x.rotate_left(4)); 
        } 

        fn bit(&mut self){
            let bits : u8 = (self.current_command & 63) >> 3;
            let reg : SingleReg = self.cpu_state.get_r8_end(self.current_command);
            let val : u8 = self.cpu_state.get_r8_val(reg); 
            self.cpu_state.set_flag(Flag::Zero, ((val>>bits)%2) == 1);
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.set_flag(Flag::HalfCarry, true); //???
            //self.cpu_state.set_flags_tri([btt((val>>bits) % 2 == 1),-1,1,0]);
        }
        fn res(&mut self){
            let bits : u8 = (self.current_command & 63) >> 3;
            let reg : SingleReg = self.cpu_state.get_r8_end(self.current_command);
            self.cpu_state.change_r8(reg, &|x| !(x & (1<<bits))); 
        } 
        fn set(&mut self){
            let bits : u8 = (self.current_command & 63) >> 3;
            let reg : SingleReg = self.cpu_state.get_r8_end(self.current_command);          
            self.cpu_state.change_r8(reg, &|x| x | 1<<bits );
        }
        fn stop(&mut self){
            loop {
                CpuStruct::wait(1);
                //PAUSE THE GPU
                //BREAK IF BUTTON PRESSED.
            }
        }
        fn halt(&mut self){
            loop { 
                CpuStruct::wait( 1) //Enter low power mode until an interrupt
            }
        }
        fn cb_block(&mut self){

            let pc = self.cpu_state.inc_pc();
            self.current_command = self.cpu_state.get_byte(pc);
            self.is_cb = true;
            let mut taken = false;
            let lookup = self.cb_block_lookup;
            let current_command = self.current_command;
            for x in lookup{
                if current_command & x.mask == x.value{
                    (x.function)(self); //Evaluate for sanity
                    taken=true;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }

            //get the next piece of memory, but we use the CB table.
        }
        pub fn wait(cycles:u8){
            //4.19 mhz * 4 t cycles 
            thread::sleep(4*CLOCK_PERIOD*cycles.into());
        }
    }
}