

pub mod cpu { 
    use crate::registers::registers::{self, SingleReg};
    use crate::registers::registers::*;
    use std::Option;
    use std::{thread,time};
    use std::time::Duration;
    use crate::memory::memory::MemoryStruct;
    const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);
    type CPUFunct = fn(&mut CpuStruct, Option<Operand>);
    enum InterruptState{
        Enabled, //Despite naming, it's really that we have E, DI, AD as "enmabled" states
        DisableInterrupt,
        AlmostDisabled,
        EnableInterrupt, 
        AlmostEnabled,
        Disabled,
    }

    pub struct CpuStruct<'a>{
        reg_set: RegStruct,
        memory_ref:&'a mut MemoryStruct,
        function_lookup:[FunFind;63],
        cb_block_lookup:[FunFind;11],
        current_command:u8,
        extra_waiting:bool,
        interrupt:InterruptState,
        is_bc:bool,
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
    enum Operand{
        SingleReg(registers::SingleReg),
        DoubleReg(registers::DoubleReg),
        //PairSingleReg(registers::SingleReg,RegStruct:SingleReg)
        MemReg(registers::DoubleReg),
        StackReg(registers::DoubleReg),
        Cond(bool),
        Bits(u8),
        BitsSingleReg(u8,registers::SingleReg),
        PairSingleReg(registers::SingleReg,registers::SingleReg),
        Imm8()
        //Imm8(u8),
        //Imm16(u16),
        //PairDoubleImm(registers::DoubleReg,u16),
    }
    impl CpuStruct<'a>{
        fn new() -> Self{
            let mmy: &mut MemoryStruct = &mut MemoryStruct::init_memory();
            Self{
                memory_ref:mmy,
                reg_set:RegStruct::build_registers(mmy),
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
                interrupt:InterruptState::Enabled
            }//Find a different way of doing this:
            //Break things apart according to our old pipeline model
        }
        fn btt (b: bool)->i8{
            -1 + (2*(b as i8))
        }
        fn get_r16(&mut self)->DoubleReg{
            self.reg_set.r16_op(self.current_command)
        }
        fn get_r8_mid(&mut self)->SingleReg{
            self.reg_set.r8_op_mid(self.current_command)
        }
        fn is_imm(&mut self)->bool{
            self.current_command & 0x11000000 > 0 
        }
        fn get_r8_end(&mut self)->SingleReg{
            if self.is_imm(){

            }
            self.reg_set.r8_op_end(self.current_command)
        }
        fn get_6_bit_arg(&mut self,ff:FunFind)->Option<Operand>{
            if ff.value == 0x40 && ff.wait==1{
                return Some(Operand::PairSingleReg(self.reg_set.r8_op_mid(ff.value), self.reg_set.r8_op_end(ff.value)))
            }
            Some(Operand::BitsSingleReg((ff.value & 127) >> 4, self.reg_set.r8_op_end(ff.value)))

        }
        fn get_mid_3_bit_arg(&mut self,ff:FunFind)->Option<Operand>{
            match ff.value {
                0xc7 => Some(Operand::Bits((ff.value & 127) >> 4)),
               _=> Some(Operand::SingleReg(self.reg_set.r8_op_mid(self.current_command)))
            }
        }
        fn get_imm8(&mut self)->u8{
            self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1))
        }
        fn get_imm16(&mut self)->u16{
            let val = self.memory_ref.grab_memory_16(self.reg_set.increment_pc(1));
            self.reg_set.increment_pc(1);
            val
        }
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            self.current_command = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1));
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
            for fun_entry in self.function_lookup{
                if self.current_command & fun_entry.mask == fun_entry.value{
                    let argument:Option<Operand> = Some(None);
                    
                    (fun_entry.function)(self,argument);
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

            //self.reg_set.increment_pc()

        }
        fn cond(&mut self)->bool{
            self.reg_set.get_cond(self.current_command)
        }
        fn alu_operand(&mut self)->SingleReg{
            if self.current_command < 0xCB00{ 
               SingleReg::A
            }else{
                self.extra_waiting = true;
                self.get_r8_end()
            }
        }
        fn nop(&mut self, arg:Option<Operand>){
            ()
        }
        // Rotates
        fn rl(&mut self, arg:Option<Operand>){
            let reg = self.alu_operand();
            let carry:bool = self.reg_set.get_flag(Flag::Carry);
            let top:bool = self.reg_set.get_bit(reg,7);
            self.reg_set.flag_cond(Flag::Carry,top);
            self.reg_set.change_single_register(reg, &|x| x<<1 + (carry as u8)); //Check to see if I need wrapping left shi
        }
        fn rr(&mut self, arg:Option<Operand>){
            let reg = self.alu_operand();
            let carry:bool = self.reg_set.get_flag(Flag::Carry);
            let bottom:bool = self.reg_set.get_bit(reg, 0);
            self.reg_set.flag_cond(Flag::Carry,bottom);
            self.reg_set.change_single_register(reg, &|x| (x>>1) + ((carry as u8)<<7));
        } 
        fn rrc(&mut self, arg:Option<Operand>){
            let reg = self.alu_operand();
            self.reg_set.flag_cond(Flag::Carry,self.reg_set.get_bit(reg, 0)); 
            self.reg_set.change_single_register(reg, &|x| x.rotate_right(1));
        }
 
        fn rlc(&mut self, arg:Option<Operand>){
            let reg = self.alu_operand();
            self.reg_set.flag_cond(Flag::Carry,self.reg_set.get_bit(reg, 7)); //Rotate right
            self.reg_set.change_single_register(reg, &|x| x.rotate_left(1));
        }
        fn daa(&mut self, arg:Option<Operand>){
            let subtract = self.reg_set.get_flag(registers::Flag::Neg);
            let hcarry = self.reg_set.get_flag(registers::Flag::HalfCarry);
            let carry = self.reg_set.get_flag(registers::Flag::Carry);
            
            //To complete
        }
        //Load Immediate
        fn ldi_r16(&mut self, arg:Option<Operand>){ //0x01
            let reg_pair: DoubleReg = self.get_r16();
            let imm2:u16 = self.get_imm16();
            self.reg_set.set_double_register(reg_pair,imm2);
            //This may actually also be like... just run str r8 imm twice.
        }
        fn ldi_r8(&mut self, arg:Option<Operand>){
            let arg_reg: SingleReg = self.get_r8_mid();
            let imm:u8 = self.get_imm8();
            self.reg_set.set_single_register(arg_reg,imm);
        }
        //Stores
        fn str_acc_rmem(&mut self, arg:Option<Operand>){//0x02
            let arg_reg: DoubleReg = self.reg_set.r16_mem(self.current_command);
            self.memory_ref.set_memory_8(self.reg_set.get_double_register(arg_reg),self.reg_set.get_acc());
        }
        fn str_c(&mut self, arg:Option<Operand>){ //Store A at address $FF00+C , e2
            let value:u16 = self.reg_set.get_register(SingleReg::C) as u16 + 0xFF00; 
            self.memory_ref.set_memory_8(value, self.reg_set.get_acc());
        } 
        fn str_imm8(&mut self, arg:Option<Operand>){ //e0 
            let reg:u8 = self.reg_set.get_acc();
            let imm:u16 = self.get_imm8() as u16;
            self.memory_ref.set_memory_8(imm+0xFF00, reg);
        }
        fn str_imm16(&mut self, arg:Option<Operand>){ //EA
            let reg:u8 = self.reg_set.get_acc();
            let imm:u16 = self.get_imm16();
            self.memory_ref.set_memory_8(imm, reg);
        }
        //Loads 
        fn ld_imm8(&mut self, arg:Option<Operand>){ //F0
            let imm:u16 = (self.get_imm8() as u16)+0xFF00;
            let mem:u8 = self.memory_ref.grab_memory_8(imm);
            self.reg_set.set_acc(mem)

        } 
        fn ld_imm16(&mut self, arg:Option<Operand>){
            let imm:u16 = self.get_imm16();
            let mem:u8 = self.memory_ref.grab_memory_8(imm);
            self.reg_set.set_acc(mem)

        }
        fn ld_imm_sp(&mut self,arg:Option<Operand>){
            let imm:u16 = self.get_imm16();
            self.reg_set.set_double_register(registers::DoubleReg::SP,imm);
        }
        fn ld_r8_r8(&mut self, arg:Option<Operand>){
            let reg_dest = self.get_r8_mid();
            let reg_src = self.get_r8_end();
            self.reg_set.set_single_register(reg_dest,self.reg_set.get_register(reg_src));
            self.extra_waiting = matches!(reg_src,SingleReg::Memptr) || !matches!(reg_dest,SingleReg::Memptr)
        }
        fn ld_acc_addr(&mut self, arg:Option<Operand>){ //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg = self.get_r16();
            self.reg_set.set_acc(self.memory_ref.grab_memory_8(self.reg_set.get_double_register(reg)));
            if matches!(reg,DoubleReg::HLP) || matches!(reg,DoubleReg::HLM) {
                self.reg_set.set_double_register(reg,0);
            }
        }
        fn ld_c(&mut self, arg:Option<Operand>){ //A = mem($FF00 + c)
            let addr:u16 = self.reg_set.get_register(SingleReg::C) as u16 + 0xFF00; 
            self.reg_set.set_acc(self.memory_ref.grab_memory_8(addr));
        } 
        fn ld_hl_imm8(&mut self, arg:Option<Operand>){ //f8
            self.reg_set.set_double_register(DoubleReg::HL, self.reg_set.get_double_register(DoubleReg::SP)+(self.get_imm8() as u16))
        }
        fn ld_sp_hl(&mut self, arg:Option<Operand>){ //f9
            self.reg_set.set_double_register(DoubleReg::HL,self.reg_set.get_double_register(DoubleReg::SP));
        }
        fn inc_r8(&mut self, arg:Option<Operand>){
            let reg:SingleReg = self.reg_set.r8_op_mid(self.current_command); //self.reg_set.r8_op_mid(self.current_command);
            self.reg_set.change_single_register(reg, &|x| x+1);
            self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        fn dec_r8(&mut self, arg:Option<Operand>){
            let reg:SingleReg = self.reg_set.r8_op_mid(self.current_command);
            self.reg_set.change_single_register(reg, &|x| x-1);
            self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        fn inc_r16(&mut self, arg:Option<Operand>){
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.change_double_register(reg_pair,&|x| x+1);
        }
        fn dec_r16(&mut self, arg:Option<Operand>){
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.change_double_register(reg_pair,&|x| x-1);
        }
        fn add_hl(&mut self, arg:Option<Operand>){
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            let double_reg_val:u16 = self.reg_set.get_double_register(reg_pair);
            self.reg_set.change_double_register(DoubleReg::HL, &|x|x+double_reg_val);
            //TODO: Flags
        }
        fn cpl(&mut self, arg:Option<Operand>){ //Invert A
            self.reg_set.change_single_register(SingleReg::A,&|x| !x);
            self.reg_set.set_flag(Flag::HalfCarry);
            self.reg_set.set_flag(Flag::Neg);
        }
        fn ccf(&mut self, arg:Option<Operand>){
            self.reg_set.flip_carry();
        }
        fn scf(&mut self, arg:Option<Operand>){
            self.reg_set.set_flag(Flag::Carry);
        }
        fn add(&mut self, arg:Option<Operand>){ 
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }
            self.reg_set.apply_fun_to_acc( &|x|x+operand)
        } 
        fn adc(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }
            let carry = self.reg_set.get_flag(Flag::Carry);
            self.reg_set.apply_fun_to_acc( &|x|x+(carry as u8)+operand)
        } 
        fn sub(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }
            self.reg_set.apply_fun_to_acc( &|x|x-operand)
        }
        fn subc(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }   
            let carry = self.reg_set.get_flag(Flag::Carry);
            self.reg_set.apply_fun_to_acc(&|x|x-(carry as u8)-operand);
        } 
        fn and(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }
            self.reg_set.apply_fun_to_acc(&|x|x&operand);
        } 
        fn xor(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }
            self.reg_set.apply_fun_to_acc( &|x|x^operand);
        } 
        fn or(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }
            self.reg_set.apply_fun_to_acc( &|x| x|operand); //what if I were to go even cooler
        }
        fn cp(&mut self, arg:Option<Operand>){
            let mut operand:u8;
            if (self.current_command & 0xC0) == 0xC0{
                operand = self.get_imm8();
            }else{
                operand = self.reg_set.get_register(self.get_r8_end());
            }                 
            let acc = self.reg_set.get_register(SingleReg::A);
            self.reg_set.set_flags((operand-acc == 0),true,true,(operand>acc)) //TODO: figure our half carry flag.
            //self.reg_set.apply_fun_to_acc(arg_reg, &|x|x&self.reg_set.get_register(process_single_arg(Operand)))
        } 

        fn ret(&mut self, arg:Option<Operand>){   
            let instruction = self.memory_ref.grab_memory_16(self.reg_set.get_double_register(DoubleReg::SP));
            self.reg_set.set_double_register(DoubleReg::PC, instruction);
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x+2)
        }
        fn ret_cond(&mut self, arg:Option<Operand>){
            if self.cond(){
                self.ret(arg);
                self.extra_waiting=true;
            }
        } 
        fn reti(&mut self, arg:Option<Operand>){
            self.ret(arg);
            self.ei(arg);
        } 
        //Jumps
        fn jr_imm(&mut self, arg:Option<Operand>){ //Jump Relative
           self.reg_set.set_pc((self.get_imm8() as u16)+self.reg_set.get_double_register(registers::DoubleReg::PC));
        }
        fn jr_cond(&mut self, arg:Option<Operand>){
            if self.cond(){
                self.jr_imm(arg);
                self.extra_waiting=true;
            } 
        }
        fn jp_cond_imm(&mut self, arg:Option<Operand>){
            if self.cond(){
                self.jp_imm(arg);
                self.extra_waiting=true;
            } 
        }  
        //Jump to 
        fn jp_imm(&mut self, arg:Option<Operand>){
            self.reg_set.set_pc(self.get_imm16());
        } 
        fn jp_hl(&mut self, arg:Option<Operand>){
            self.reg_set.set_double_register(registers::DoubleReg::PC, self.reg_set.get_double_register(registers::DoubleReg::HL))
        } 
        fn call_cond(&mut self, arg:Option<Operand>){
            if self.cond(){
                self.call_imm(arg);
                self.extra_waiting=true;
            } 
        } 
        fn call_imm(&mut self, arg:Option<Operand>){
            let addr = self.get_imm16();
            self.memory_ref.set_memory_16(self.reg_set.get_double_register(DoubleReg::SP), self.reg_set.get_double_register(DoubleReg::PC));
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x-2);
            self.reg_set.set_pc(addr);
        } 
        fn rst(&mut self, arg:Option<Operand>){
            let value: u16 = self.reg_set.get_double_register(DoubleReg::PC);
            self.memory_ref.set_memory_16(self.reg_set.get_double_register(DoubleReg::SP), value);
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x-2);
            self.reg_set.set_pc((self.current_command & 0x00111000) as u16);
        }
        fn pop(&mut self, arg:Option<Operand>){
            let operand = self.reg_set.r16_stk(self.current_command);
            let instruction = self.memory_ref.grab_memory_16(self.reg_set.get_double_register(DoubleReg::SP));
            self.reg_set.set_double_register(operand, instruction);
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x+2)
        }
        fn push(&mut self, arg:Option<Operand>){
            let operand = self.reg_set.r16_stk(self.current_command);
            let value: u16 = self.reg_set.get_double_register(operand);
            self.memory_ref.set_memory_16(self.reg_set.get_double_register(DoubleReg::SP), value);
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x-2)
        }
        fn add_sp_imm8(&mut self, arg:Option<Operand>){
            let operand:i8 = self.get_imm8() as i8;
            self.reg_set.change_double_register(registers::DoubleReg::SP, &|x| x.wrapping_add_signed(operand as i16));
        }

        fn di(&mut self, arg:Option<Operand>){
            self.interrupt =match self.interrupt{
                InterruptState::AlmostDisabled|InterruptState::Disabled => InterruptState::Disabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::AlmostDisabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        fn ei(&mut self, arg:Option<Operand>){
            self.interrupt =match self.interrupt{
                InterruptState::AlmostDisabled|InterruptState::Disabled => InterruptState::AlmostEnabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        } 

        fn sla(&mut self, arg:Option<Operand>){
            let reg = self.get_r8_end();
            let operand = self.reg_set.get_register(reg);                
            self.reg_set.reset_flags();
            self.reg_set.flag_cond(Flag::Carry, operand>127);
            self.reg_set.flag_cond(Flag::Zero, operand == 128 || operand==0);
            self.reg_set.change_single_register(reg, &|x|x<<1);
        } 
        fn sra(&mut self, arg:Option<Operand>){
            let reg = self.get_r8_end();
            let operand = self.reg_set.get_register(reg);
            self.reg_set.reset_flags();
            self.reg_set.flag_cond(Flag::Zero,operand==1 || operand == 0 );
            self.reg_set.flag_cond(Flag::Carry,operand%2 == 1 );
            self.reg_set.change_single_register(reg, &|x|(x>>1)+(128 * ((x>127) as u8))); //Sneaky little arithmetic right shift.
        } 
        fn srl(&mut self, arg:Option<Operand>){
            let reg = self.get_r8_end();
            let operand = self.reg_set.get_register(reg);
            self.reg_set.reset_flags();
            self.reg_set.flag_cond(Flag::Zero,operand==1 || operand == 0 );
            self.reg_set.flag_cond(Flag::Carry,operand%2 == 1 );
            self.reg_set.change_single_register(reg, &|x|(x>>1)); 
        } 
        fn swap(&mut self, arg:Option<Operand>){
            let reg = self.get_r8_end();
            let operand = self.reg_set.get_register(reg);
            self.reg_set.reset_flags();
            self.reg_set.flag_cond(Flag::Zero,operand==0);
            self.reg_set.change_single_register(reg, &|x:u8|x.rotate_left(4)); 
        } 

        fn bit(&mut self, arg:Option<Operand>){
            let bits : u8 = ((self.current_command & 63) >> 3);
            let reg : SingleReg = self.get_r8_end();
            let val : u8 = self.reg_set.get_register(reg); 
            self.reg_set.set_flags_tri([btt((val>>bits) % 2 == 1),-1,1,0]);
        }
        fn res(&mut self, arg:Option<Operand>){
            let bits : u8 = (self.current_command & 63) >> 3;
            let reg : SingleReg = self.get_r8_end();
            self.reg_set.change_single_register(reg, &|x| !(x & (1<<bits))); 
        } 
        fn set(&mut self, arg:Option<Operand>){
            let bits : u8 = (self.current_command & 63) >> 3;
            let reg : SingleReg = self.get_r8_end();          
            self.reg_set.change_single_register(reg, &|x| x | 1<<bits );
        }
        fn stop(&mut self, arg:Option<Operand>){
            loop {
                CpuStruct::wait(1);
                //PAUSE THE GPU
                //BREAK IF BUTTON PRESSED.
            }
        }
        fn halt(&mut self, arg:Option<Operand>){
            loop { 
                CpuStruct::wait( 1) //Enter low power mode until an interrupt
            }
        }
        fn cb_block(&mut self, arg:Option<Operand> ){
            let local_command = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1));
            self.current_command = 0xcb00+ (local_command as u16);
            let mut taken = false;
            for x in self.cb_block_lookup{
                if local_command & x.mask == x.value{
                    (x.function)(self,arg); //Evaluate for sanity
                    taken=true;
                }
            }
            if !taken{
                panic!("we didn't do anything!")
            }

            //grab the next piece of memory, but we use the CB table.
        }
        pub fn wait(cycles:u8){
            //4.19 mhz * 4 t cycles 
            thread::sleep(4*CLOCK_PERIOD*cycles.into());
        }
    }
}