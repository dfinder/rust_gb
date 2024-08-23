

pub mod cpu { 
    use crate::registers::registers::{self, SingleReg};
    use crate::registers::registers::*;
    use crate::cpu_state::cpu_state::*;
    use crate::function_table::function_table::FunFind;
    use std::{thread,time};
    use std::time::Duration;
    const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);
    use crate::interrupt::interrupt::InterruptType;
    pub type CPUFunct =  fn(&mut CpuStruct);
    enum InterruptState{
        Enabled, //Despite naming, it's really that we have E, DI, AD as "enmabled" states
        DisableInterrupt, 
        //AlmostDisabled, //As it turns out, DI is instant.
        EnableInterrupt, 
        AlmostEnabled,
        Disabled,
    }
    pub struct CpuStruct{
        pub cpu_state: CpuState,
        function_lookup:[FunFind;63],
        cb_block_lookup:[FunFind;11],
        instruction_register:u8,
        extra_waiting:bool,
        ime_flag:InterruptState,
        stopped:bool,
        halted:bool
        //Preprocess Option<Operand>
        //used for mem reads to HL, failed conditional jumps
        //argument:Argument;
    }
    impl CpuStruct{
        pub fn new() -> Self{
            Self{
                cpu_state: CpuState::new(),
                instruction_register:0x00, //initalize to a noop
                function_lookup:[
                    //Block 1,
                    FunFind::fun_find(0xff,0x00,CpuStruct::nop,1), //done //Reasses waits.
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
                ime_flag:InterruptState::DisableInterrupt, //Interreupt master enable
                stopped: false,
                halted: false
            }//Find a different way of doing this:
            //Break things apart according to our old pipeline model
        }
        pub fn fetch_graphics(&mut self)->&[u8;8192]{
            self.cpu_state.get_graphics()
        }
        pub fn interpret_command(&mut self){ //function_lookup:&[FunFind;63], cb_lookup:&[FunFind;11]
            if !self.stopped{ //TODO: Fetch is actually the last part of the instruction, so the PC counter is  always one ahead of the actual instruction
                let current_pc = self.cpu_state.inc_pc();
                self.instruction_register = self.cpu_state.get_byte(current_pc);
                let mut taken:bool = false;
                self.extra_waiting = false;
                if self.instruction_register != 0xF3{ //DI I think we effectively handle this twice.
                    self.handle_interrupts();
                }
                let mut fun_pointer: fn(&mut CpuStruct) = CpuStruct::nop;
                let mut waiting = 0;
                let mut cond_waiting = 0;
                for fun_entry in &self.function_lookup{
                    if (self.instruction_register & fun_entry.mask) == fun_entry.value{
                            fun_pointer=fun_entry.function;
                            waiting=fun_entry.wait;
                            if fun_entry.wait_cond.is_some(){
                                cond_waiting=fun_entry.wait_cond.unwrap();
                            }
                            taken = true;
                    }
                }
                if !taken{
                    panic!("we didn't do anything!")
                }
                fun_pointer(self);
                if self.extra_waiting{
                    CpuStruct::wait(waiting);
                }else{
                    CpuStruct::wait(cond_waiting);
                }; //Evaluate for sanity
            }else{
                self.handle_interrupts();
                CpuStruct::wait(1); 
            }
            //self.interpret_command(function_lookup, cb_lookup)
            //Manage interrupts
        }
        pub fn cond(&mut self)->bool{
            self.cpu_state.get_cond(self.instruction_register)
        }
        pub fn alu_register(&mut self)->SingleReg{
            if self.instruction_register > 0b11000000 { 
                SingleReg::A
            }else{
                self.cpu_state.get_r8_end(self.instruction_register)
            }
        }
        pub fn alu_operand(&mut self)->u8{
            if self.instruction_register >0b11000000 { 
               self.cpu_state.get_imm8()
            }else{
                self.extra_waiting = true;
                let register= self.cpu_state.get_r8_end(self.instruction_register);
                self.cpu_state.get_r8_val(register)
            }
        }
        pub fn nop(&mut self){
            ()
        }
        // Rotations
        pub fn rl(&mut self){
            let val = self.alu_operand();
            let reg = self.alu_register();
            let carry:bool = self.cpu_state.get_flag(Flag::Carry);
            self.cpu_state.change_r8(reg, &|x| x<<1 + (carry as u8)); //Check to see if I need wrapping left shi
            self.cpu_state.set_flag(Flag::Carry,val>127);
        }
        pub fn rlc(&mut self){ //Rotate left
            let val = self.alu_operand();
            let reg = self.alu_register();
            self.cpu_state.set_flag(Flag::Carry,val>127); 
            self.cpu_state.change_r8(reg, &|x| x.rotate_left(1));
        }
        pub fn rr(&mut self){
            let reg = self.alu_register();
            let val = self.alu_operand();
            let carry:bool = self.cpu_state.get_flag(Flag::Carry);
            let bottom:bool = (val % 2)==1;
            self.cpu_state.set_flag(Flag::Carry,bottom);
            self.cpu_state.change_r8(reg, &|x| (x>>1) + ((carry as u8)<<7));
        } 
        pub fn rrc(&mut self){
            let value = self.alu_operand();
            let reg = self.alu_register();
            self.cpu_state.set_flag(Flag::Carry,(value % 2)==1); 
            self.cpu_state.change_r8(reg, &|x| x.rotate_right(1));
        }

        pub fn daa(&mut self){
            let subtract = self.cpu_state.get_flag(registers::Flag::Neg);
            let hcarry = self.cpu_state.get_flag(registers::Flag::HalfCarry);
            let carry = self.cpu_state.get_flag(registers::Flag::Carry);
            let mut offset:u8= 0;
            let a_val = self.cpu_state.get_acc();
            if (!subtract && a_val&0xf > 0x9) || hcarry{
                offset |= 0x06;
            }
            if (!subtract && a_val > 0x99) || carry{
                offset |= 0x60;
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
            self.cpu_state.set_flag(Flag::Carry, acc>0x99);
        }
        //Load Immediate
        pub fn ldi_r16(&mut self){ //0x01
            let reg_pair: DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            let imm2:u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(reg_pair,imm2);
            //This may actually also be like... just run str r8 imm twice.
        }
        pub fn ldi_r8(&mut self){
            let arg_reg: SingleReg = self.cpu_state.get_r8_mid(self.instruction_register);
            let imm:u8 = self.cpu_state.get_imm8();
            self.cpu_state.set_r8(arg_reg,imm);
        }
        //Stores
        pub fn str_acc_rmem(&mut self){//0x02
            let arg_reg: DoubleReg = self.cpu_state.r16_mem_tbl(self.instruction_register);
            let mem_addr: u16 = self.cpu_state.get_r16_val(arg_reg);
            let acc_val = self.cpu_state.get_acc();
            self.cpu_state.set_byte(mem_addr,acc_val);
        }
        pub fn str_c(&mut self){ //Store A at address $FF00+C , e2
            let value:u16 = self.cpu_state.get_r8_val(SingleReg::C) as u16 + 0xFF00; 
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_byte(value, acc );
        } 
        pub fn str_imm8(&mut self){ //e0 
            let reg:u8 = self.cpu_state.get_acc();
            let imm:u16 = self.cpu_state.get_imm8() as u16;
            self.cpu_state.set_byte(imm+0xFF00, reg);
        }
        pub fn str_imm16(&mut self){ //EA
            let reg:u8 = self.cpu_state.get_acc();
            let imm:u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_byte(imm, reg);
        }
        //Loads 
        pub fn ld_imm8(&mut self){ //F0
            let imm:u16 = (self.cpu_state.get_imm8() as u16)+0xFF00;
            let mem:u8 = self.cpu_state.get_byte(imm);
            self.cpu_state.set_acc(mem)

        } 
        pub fn ld_imm16(&mut self){
            let imm:u16 = self.cpu_state.get_imm16();
            let mem:u8 = self.cpu_state.get_byte(imm);
            self.cpu_state.set_acc(mem)
        }
        pub fn ld_imm_sp(&mut self){
            let imm:u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(registers::DoubleReg::SP,imm);
        }
        pub fn ld_r8_r8(&mut self){
            let reg_dest = self.cpu_state.get_r8_mid(self.instruction_register);
            let reg_src = self.cpu_state.get_r8_end(self.instruction_register);
            let reg_val = self.cpu_state.get_r8_val(reg_src);
            self.cpu_state.set_r8(reg_dest,reg_val);
            self.extra_waiting = !matches!(reg_dest,SingleReg::Memptr)
        }
        pub fn ld_acc_addr(&mut self){ //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg = self.cpu_state.r16_tbl(self.instruction_register);
            let mem: u8 = self.cpu_state.get_r16_memory(reg);
            self.cpu_state.set_acc(mem);
            if matches!(reg,DoubleReg::HLP) || matches!(reg,DoubleReg::HLM) {
                self.cpu_state.set_r16_val(reg,0);
            }
        }
        pub fn ld_c(&mut self){ //A = mem($FF00 + c)
            let addr:u16 = self.cpu_state.get_r8_val(SingleReg::C) as u16 + 0xFF00; 
            let value:u8 = self.cpu_state.get_byte(addr);
            self.cpu_state.set_acc(value);
        } 
        pub fn ld_hl_imm8(&mut self){ //f8
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let imm = self.cpu_state.get_imm8() as u16;
            self.cpu_state.set_r16_val(DoubleReg::HL, stack_pointer+imm);
        }
        pub fn ld_sp_hl(&mut self){ //f9
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            self.cpu_state.set_r16_val(DoubleReg::HL,stack_pointer);
        }
        pub fn inc_r8(&mut self){
            let reg:SingleReg = self.cpu_state.get_r8_mid(self.instruction_register); //self.cpu_state.r8_op_mid(self.instruction_register);
            let val:u8 = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, val==0xff);
            self.cpu_state.set_flag(Flag::HalfCarry, val==0x0f);
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.change_r8(reg, &|x| x.wrapping_add(1));
            
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        pub fn dec_r8(&mut self){
            let reg:SingleReg = self.cpu_state.get_r8_mid(self.instruction_register);
            let val:u8 = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, val==0x01);
            self.cpu_state.set_flag(Flag::HalfCarry, val==0x10);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.change_r8(reg, &|x| x.wrapping_sub(1));
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        pub fn inc_r16(&mut self){ //Doesn't affect flags
            let reg_pair:DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            self.cpu_state.change_r16(reg_pair,&|x| x+1);
        }
        pub fn dec_r16(&mut self){ //doesn't affect flags.
            let reg_pair:DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            self.cpu_state.change_r16(reg_pair,&|x| x-1);
        }
        pub fn add_hl(&mut self){
            let reg_pair:DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            let operand:u16 = self.cpu_state.get_r16_val(reg_pair);
            let hl_val:u16 = self.cpu_state.get_r16_val(DoubleReg::HL);
            let result = self.cpu_state.change_r16(DoubleReg::HL, &|x|x.wrapping_add(operand));
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.set_flag(Flag::HalfCarry, (hl_val&0x0fff)+(operand & 0x0fff)>0x1000);
            self.cpu_state.set_flag(Flag::Carry, None == hl_val.checked_add(operand));
            self.cpu_state.set_flag(Flag::Zero,result==0);
        }
        pub fn cpl(&mut self){ //Invert A
            self.cpu_state.set_flag(Flag::HalfCarry,true);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.change_r8(SingleReg::A,&|x| !x);
        }
        pub fn ccf(&mut self){
            self.cpu_state.flip_carry();
        }
        pub fn scf(&mut self){
            self.cpu_state.set_flag(Flag::Carry,true);
        }
        pub fn add(&mut self){ 
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            let ret = self.cpu_state.apply_fun_to_acc( &|x|x.wrapping_add(operand));
            self.cpu_state.set_flag(Flag::Zero, ret==0);
            self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand) );
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F)) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg,false);
        } 
        pub fn adc(&mut self){
            let carry = self.cpu_state.get_flag(Flag::Carry);
            let operand = self.alu_operand()+(carry as u8);
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc.wrapping_add(operand)==0);
            self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand));
            self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F) as u8) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.apply_fun_to_acc( &|x|x.wrapping_add(operand));
        } 
        pub fn sub(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==operand);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.set_flag(Flag::HalfCarry, (acc & 0x0F)<(operand & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand>acc);
            self.cpu_state.apply_fun_to_acc( &|x|x.wrapping_sub(operand));
        }
        pub fn subc(&mut self){
            let carry:u8 = self.cpu_state.get_flag(Flag::Carry) as u8;
            let operand: u8 = self.alu_operand()+carry;
            let acc: u8 = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==operand);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.set_flag(Flag::HalfCarry, (acc & 0x0F)<((operand) & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand>acc);
            self.cpu_state.apply_fun_to_acc(&|x|x.wrapping_sub(operand));
        } 
        pub fn and(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flags(operand&acc == 0, false, true, false);
            self.cpu_state.apply_fun_to_acc(&|x|x&operand);
        } 
        pub fn xor(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flags(operand^acc == 0 , false, true, false);
            self.cpu_state.apply_fun_to_acc( &|x|x^operand);
        } 
        pub fn or(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flags(operand|acc == 0 , false, true, false);
            self.cpu_state.apply_fun_to_acc( &|x| x|operand); //what if I were to go even cooler
        }
        pub fn cp(&mut self){
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc==operand);
            self.cpu_state.set_flag(Flag::Neg,true);
            self.cpu_state.set_flag(Flag::HalfCarry, (acc & 0x0F)<(operand & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand>acc);
        } 

        pub fn ret(&mut self){   
            let instruction = self.cpu_state.get_r16_memory_word(DoubleReg::SP);
            self.cpu_state.set_r16_val(DoubleReg::PC, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x+2);
        }
        pub fn ret_cond(&mut self){
            if self.cond(){
                self.ret();
                self.extra_waiting=true;
            }
        } 
        pub fn reti(&mut self){
            self.ret();
            self.ei();
        } 
        //Jumps
        pub fn jr_imm(&mut self){ //Jump Relative
            let imm = self.cpu_state.get_imm8() as i16;
            let pc = self.cpu_state.get_r16_val(registers::DoubleReg::PC);
            self.cpu_state.set_pc(pc.wrapping_add_signed(imm));
        }
        pub fn jr_cond(&mut self){
            if self.cond(){
                self.jr_imm();
                self.extra_waiting=true;
            } 
        }
        pub fn jp_cond_imm(&mut self){
            if self.cond(){
                self.jp_imm();
                self.extra_waiting=true;
            } 
        }  
        pub fn jp_imm(&mut self){
            let imm: u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(registers::DoubleReg::PC, imm);
        } 
        pub fn jp_hl(&mut self){
            let hl: u16 =  self.cpu_state.get_r16_val(registers::DoubleReg::HL);
            self.cpu_state.set_r16_val(registers::DoubleReg::PC,hl)
        } 
        pub fn call_cond(&mut self){
            if self.cond(){
                self.call_imm();
                self.extra_waiting=true;
            } 
        } 
        pub fn call_imm(&mut self){
            let addr = self.cpu_state.get_imm16();
            let pc = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_r16_memory(DoubleReg::SP, pc);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
            self.cpu_state.set_pc(addr);
        } 
        pub fn rst(&mut self){
            let pc: u16 = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_r16_memory(DoubleReg::SP, pc);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
            self.cpu_state.set_pc((self.instruction_register & 0b00111000) as u16);
        }
        pub fn pop(&mut self){
            let operand = self.cpu_state.r16_stk_tbl(self.instruction_register);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let instruction = self.cpu_state.get_half_word(stack_pointer);
            self.cpu_state.set_r16_val(operand, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x+2);
        }
        pub fn push(&mut self){
            let operand = self.cpu_state.r16_stk_tbl(self.instruction_register);
            let value: u16 = self.cpu_state.get_r16_val(operand);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);

            self.cpu_state.set_half_word(stack_pointer, value);
            self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
        }
        pub fn add_sp_imm8(&mut self){
            let operand:i8 = self.cpu_state.get_imm8() as i8;
            //self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand) );
            //self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F)) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg,false);
            self.cpu_state.set_flag(Flag::Zero,false);
            self.cpu_state.change_r16(registers::DoubleReg::SP, &|x| x.wrapping_add_signed(operand as i16));
        }

        pub fn di(&mut self){
            self.ime_flag =match self.ime_flag{
                InterruptState::Disabled => InterruptState::Disabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::DisableInterrupt,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        pub fn ei(&mut self){
            self.ime_flag =match self.ime_flag{
                InterruptState::Disabled => InterruptState::EnableInterrupt,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        } 
        pub fn sla(&mut self){
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);                
            self.cpu_state.set_flags(operand == 128 || operand==0,false,false,operand>127);
            self.cpu_state.change_r8(reg, &|x|x<<1);
        } 
        pub fn sra(&mut self){
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand == 1 || operand==0,false,false,operand%2 == 1);
            self.cpu_state.change_r8(reg, &|x|(x>>1)+(128 * ((x>127) as u8))); //Sneaky little arithmetic right shift.
        } 
        pub fn srl(&mut self){
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand == 1 || operand==0,false,false,operand%2 == 1);
            self.cpu_state.change_r8(reg, &|x|(x>>1)); 
        } 
        pub fn swap(&mut self){
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand==0,false,false,false);
            self.cpu_state.change_r8(reg, &|x:u8|x.rotate_left(4)); 
        } 
        pub fn bit(&mut self){
            let bits : u8 = (self.instruction_register & 63) >> 3;
            let reg : SingleReg = self.cpu_state.get_r8_end(self.instruction_register);
            let val : u8 = self.cpu_state.get_r8_val(reg); 
            self.cpu_state.set_flag(Flag::Zero, ((val>>bits)%2) == 1);
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.set_flag(Flag::HalfCarry, true); //???x
        }
        pub fn res(&mut self){
            let bits : u8 = (self.instruction_register & 63) >> 3;
            let reg : SingleReg = self.cpu_state.get_r8_end(self.instruction_register);
            self.cpu_state.change_r8(reg, &|x| !(x & (1<<bits))); 
        } 
        pub fn set(&mut self){
            let bits : u8 = (self.instruction_register & 63) >> 3;
            let reg : SingleReg = self.cpu_state.get_r8_end(self.instruction_register);          
            self.cpu_state.change_r8(reg, &|x| x | 1<<bits );
        }
        //pub fn unstop(&mut self){ //Not actually a cpu command.
        //    self.stopped=false;
        //}
        pub fn stop(&mut self){ //No officialrom uses stop, so we're using the stop flag for halt instead
            self.stopped=true;
        }
        pub fn halt(&mut self){ //We need to implement the halt bug where we repeat the PC counter.
            self.stopped=true;
            self.ime_flag =InterruptState::Enabled;
        }
        pub fn cb_block(&mut self){ 
            let pc = self.cpu_state.inc_pc();
            self.instruction_register = self.cpu_state.get_byte(pc);
            let mut fun_pointer: fn(&mut CpuStruct)=CpuStruct::nop;
            let mut waiting= 0;
            let mut cond_waiting = 0;
            let mut taken = false;
            for fun_entry in &self.cb_block_lookup{
                if (self.instruction_register & fun_entry.mask) == fun_entry.value{
                        fun_pointer=fun_entry.function;
                        waiting=fun_entry.wait;
                        if fun_entry.wait_cond.is_some(){
                            cond_waiting=fun_entry.wait_cond.unwrap();
                        }
                        taken = true;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }
            fun_pointer(self);
            if self.extra_waiting{
                CpuStruct::wait(waiting);
            }else{
                CpuStruct::wait(cond_waiting);
            }; //Evaluate for sanity
            if !taken{
                panic!("we didn't do anything!")
            }
            //get the next piece of memory, but we use the CB table.
        }
        pub fn wait(cycles:u8){
            //4.19 mhz * 4 t cycles 
            thread::sleep(4*CLOCK_PERIOD*cycles.into());
        }
        pub fn handle_interrupts(&mut self){
            self.ime_flag = match self.ime_flag{ //Handle the transition of the IME flag.
                InterruptState::Enabled | InterruptState::AlmostEnabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => InterruptState::AlmostEnabled,
                InterruptState::DisableInterrupt|InterruptState::Disabled => InterruptState::Disabled
            };
            let are_interreupts_enabled:bool = match self.ime_flag{
                InterruptState::Enabled|InterruptState::DisableInterrupt => true,
                _ => false
            };
            let enabled_interrupts_flag:u8 = self.cpu_state.get_byte(0xFFFF);
            let mut bit_idx:u8 = 1;
            let mut target_call:u16 = 0x0040;
            let interrupt_flag:u8 = self.cpu_state.get_byte(0xFF0F);
            loop{
                if (interrupt_flag & enabled_interrupts_flag & bit_idx) > 0{ //We process an interrupt
                    if are_interreupts_enabled{ //We check in here so that if we have the case where we're halted, and don't have IME Enabled, we unhalt
                        self.cpu_state.set_byte(0xFF0F, interrupt_flag ^ bit_idx); //unset bit 
                        self.ime_flag = InterruptState::Disabled;
                        let pc = self.cpu_state.get_r16_val(DoubleReg::PC); //CALL
                        self.cpu_state.set_r16_memory(DoubleReg::SP, pc);
                        self.cpu_state.change_r16(DoubleReg::SP, &|x|x-2);
                        self.cpu_state.set_pc(target_call);
                        CpuStruct::wait(5);
                    }
                    self.halted=false; //Implement the Halt bug!
                    break;
                }
                bit_idx*=2;
                target_call+=8;
                if bit_idx == 0x20{
                    break
                }
            }
                //Vblank has the highest priority
            
        }
        pub fn interrupt(&mut self, interrupt:InterruptType){
            match self.ime_flag{
                InterruptState::Enabled | InterruptState::DisableInterrupt => (),
                _ => return ()
            };
            let bit_idx: u8 = match interrupt{
                InterruptType::VBlank => 0,
                InterruptType::LCDC => 1,
                InterruptType::Timer => 2,
                InterruptType::Serial => 3,
                InterruptType::Input => 4
            };
            let mut current_interrupt_flag: u8 =  self.cpu_state.get_byte(0xFF0F);
            current_interrupt_flag |= 1<<bit_idx;
            self.cpu_state.set_byte(0xFF0F, current_interrupt_flag);
        }

    }
}