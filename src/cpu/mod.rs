

pub mod cpu { 
    use crate::registers::registers::{self, SingleReg};
    use crate::registers::registers::*;
    use std::option;
    use std::{thread,time};
    use std::time::Duration;
    use crate::memory::memory::MemoryStruct;
    const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);
    type CPUFunct = fn(&mut CpuStruct, Args);
    enum InterruptState{
        Enabled, //Despite naming, it's really that we have E, DI, AD as "enmabled" states
        DisableInterrupt,
        AlmostDisabled,
        EnableInterrupt, 
        AlmostEnabled,
        Disabled,
    }

    pub struct CpuStruct{
        reg_set: RegStruct,
        memory_ref:&'static MemoryStruct,
        function_lookup:[FunFind;63],
        cb_block_lookup:[FunFind;11],
        current_command:u8,
        extra_waiting:bool,
        interrupt:InterruptState
        //Preprocess Args
        //used for mem reads to HL, failed conditional jumps
        //argument:Argument;
    }
    pub struct FunFind{
        mask:u8,
        value:u8,
        function: CPUFunct ,//,argument Arg. returns false if we have a longer wait.
        wait:u8,
        wait_cond:Option<u8>,
        arg: Option<RegArg>,
        imm: u8
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
                arg:None,
                imm:1
            }
        }
        fn fun_find_a(mask: u8, value: u8, fun:fn(&mut CpuStruct, Args), wait:u8, arg:Operand)->Self{
            Self{
                mask: mask,
                value:value,
                function:fun,
                wait:wait,
                wait_cond: Some(wait_cond),
                arg: arg,
                imm: 1
            }
        }
        fn fun_find_w(mask: u8, value: u8, fun:fn(&mut CpuStruct, Args), wait:u8, wait_cond:u8 )->Self{
            Self{
                mask: mask,
                value:value,
                function:fun,
                wait:wait,
                wait_cond: Some(wait_cond),
                arg: None,
                imm: 1
            }
        }
    }
     
    //type Arg= (Operand,Imm);
    enum Imm {
        Imm8(u8),
        Imm16(u16),
    }
    pub struct Args {
        op: Option<Operand>,
        imm: Option<Imm>
    }
    impl Args {
        pub fn make_arg(op:Operand,imm:Imm)->Self{
            Self{
                op: Some(op),
                imm: Some(imm)
            }
        }
        fn make_op(op:Operand)->Self{
            Self{
                op: Some(op),
                imm: None,
            }
        }
        fn get_op(&mut self)->Optional<Operand>{
            self.op
        }
        fn get_imm(&mut self)->Optional<Imm>{
            self.imm
        }
    }
    enum Operand{
        SingleRegArg(registers::SingleReg),
        DoubleRegArg(registers::DoubleReg),
        //PairSingleReg(registers::SingleReg,RegStruct:SingleReg)
        MemReg(registers::DoubleReg),
        StackReg(registers::DoubleReg),
        Cond(bool),
        Bits(u8),
        PairSingleReg(registers::SingleReg,registers::SingleReg),
        //Imm8(u8),
        //Imm16(u16),
        //PairDoubleImm(registers::DoubleReg,u16),
        BitsSingleReg(u8,registers::SingleReg),
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
                    FunFind::fun_find(0xcf,0x02,Self::str_r16_acc,2),
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
                    FunFind::fun_find_w(0xf8,0x80,Self::add,1,2),
                    FunFind::fun_find_w(0xf8,0x88,Self::adc,1,2),
                    FunFind::fun_find_w(0xf8,0x90,Self::sub,1,2),
                    FunFind::fun_find_w(0xf8,0x98,Self::subc,1,2),
                    FunFind::fun_find_w(0xf8,0xa0,Self::and,1,2),
                    FunFind::fun_find_w(0xf8,0xa8,Self::xor,1,2),
                    FunFind::fun_find_w(0xf8,0xb0,Self::or,1,2),
                    FunFind::fun_find_w(0xf8,0xb8,Self::cp,1,2),
                    FunFind::fun_find_w(0xff,0xc6,Self::add,1,2),
                    FunFind::fun_find_w(0xff,0xce,Self::adc,1,2),
                    FunFind::fun_find_w(0xff,0xd6,Self::sub,1,2),
                    FunFind::fun_find_w(0xff,0xde,Self::subc,1,2),
                    FunFind::fun_find_w(0xff,0xe6,Self::and,1,2),
                    FunFind::fun_find_w(0xff,0xee,Self::xor,1,2),
                    FunFind::fun_find_w(0xff,0xf6,Self::or,1,2),
                    FunFind::fun_find_w(0xff,0xfe,Self::cp,1,2),
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
                    FunFind::fun_find_w(0xf8,0x00,Self::rlc,2,4),
                    FunFind::fun_find_w(0xf8,0x08,Self::rrc,2,4),
                    FunFind::fun_find_w(0xf8,0x10,Self::rl,2,4),
                    FunFind::fun_find_w(0xf8,0x18,Self::rr,2,4),
                    FunFind::fun_find_w(0xf8,0x20,Self::sla,2,4),
                    FunFind::fun_find_w(0xf8,0x28,Self::sra,2,4),
                    FunFind::fun_find_w(0xf8,0x30,Self::swap_r8,2,4),
                    FunFind::fun_find_w(0xf8,0x38,Self::srl_r8,2,4),
                    FunFind::fun_find_w(0xc0,0x40,Self::bit,2,3),
                    FunFind::fun_find_w(0xc0,0x80,Self::res,2,4),
                    FunFind::fun_find_w(0xc0,0x90,Self::set,2,4)
                ],
                extra_waiting:false,
                interrupt:InterruptState::Enabled
            }//Find a different way of doing this:
            //Break things apart according to our old pipeline model
        }
        fn get_double_register_from_opcode(&mut self,ff: FunFind)->Option<Operand>{
            match ff.function{
                Self::str_r16_imm | Self::inc_r16 | Self::dec_r16 => Some(Operand::DoubleRegArg(self.reg_set.r16_op(ff.value))),
                Self::str_addr_acc | Self::ld_acc_addr => Some(Operand::MemReg(self.reg_set.r16_mem(ff.value))),
                Self::push | Self::pop => Some(Operand::StackReg(self.reg_set.r16_stk(ff.value))),
                _ => None
            }
        }
        fn get_6_bit_arg(&mut self,ff:FunFind)->Operand{
            match ff.function{
                Self::bit | Self::res | self::set => Operand::BitsSingleReg((ff.value & 127) >> 4, self.reg_set.r8_op_end(ff.value)), //Clear bit 7, take bits 6-4 -> 2->0
                Self::ld_r8_r8 => Operand::PairSingleReg(self.reg_set.r8_op_mid(ff.value), self.reg_set.r8_op_end(ff.value))
            }
        }
        fn get_imm(&mut self,ff:FunFind)->Option<Imm>{
            match ff.imm{
                1 => Empty,
                2 => Some(Imm8(self.grab_imm8())),
                3 => Some(Imm16(self.grab_imm16())),
            }
        }
        fn get_mid_3_bit_arg(&mut self,ff:FunFind)->Option<Operand>{
            match ff.function {
            self::rst => Some(Operand::Bits((ff.value & 127) >> 4)),
               _=> Some(Operand::SingleRegArg(self.reg_set.r8_op_mid(current_command)))
            }
        }
        fn grab_imm8(&mut self)->u8{
            self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1))
        }
        fn grab_imm16(&mut self)->u16{
            self.memory_ref.grab_memory_16(self.reg_set.increment_pc(1))
        }
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            let current_command:u8 = self.memory_ref.grab_memory_8(self.reg_set.increment_pc(1));
            //let first_two:u8 = current_command >> 6
            //static masks:[u8]=[0xFF,0xCF,0xE7,0xC0,0xC7,0xF8]
            let mut taken:bool = false;
            self.waiting = 0;
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
                if current_command & fun_entry.mask == fun_entry.value{
                    let argument:Option<Operand> = match fun_entry.mask{
                        0xff => None, 
                        0xcf => self.get_double_register_from_opcode(fun),
                        0xe7 => Some(Operand::Cond(self.reg_set.get_cond(current_command))),
                        0xc0 => self.get_6_bit_arg(fun),
                        0xc7 => self.get_mid_3_bit_arg(fun),
                        0xf8 => Some(Operand::SingleRegArg(self.reg_set.r8_op_end(current_command))),
                        _ => unreachable!()
                        //let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
                    };
                    let imm:Option<Imm> = self.get_imm(fun);
                    (fun_entry.function)(self,Args::make_arg(argument, imm));
                    if (self.waiting){
                        self.wait(fun_entry.wait);

                    }else{
                        self.wait(fun_entry.wait_cond.unwrap());
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
        
        fn nop(&mut self, arg:Args)->bool{
            ;
        }
        // Rotates
        fn rl(&mut self, arg:Args){
            let arg_reg = match arg.op{
                Some(Operand::SingleRegArg(reg)) => reg,
                _ => unreachable!()
            };
            let carry:bool = self.reg_set.get_flag(Flag::Carry);
            let top:bool = self.reg_set.get_bit(SingleReg::A,7);
            if top{
                self.reg_set.set_flag(Flag::Carry);
            }
            self.reg_set.change_single_register(arg_reg, &|x| x<<1 + (carry as u8)); //Check to see if I need wrapping left shi
        }
        fn rr(&mut self, arg:Args){
            let arg_reg = match arg.get_op(){
                Operand::SingleRegArg(reg) => reg,
                _ => unreachable!()
            };
            let carry:bool = self.reg_set.get_flag(Flag::Carry);
            let bottom:bool = self.reg_set.get_bit(registers::SingleReg::A, 0);
            if bottom{
                self.reg_set.set_flag(Flag::Carry);
            }
            self.reg_set.change_single_register(arg_reg, &|x| (x>>1) + ((carry as u8)<<7));
        } 
        fn rrc(&mut self, arg:Args){
            let arg_reg = match arg.get_op(){
                Operand::SingleRegArg(reg) => reg,
                _ => unreachable!()
            };
            if self.reg_set.get_bit(arg_reg, 0){
                self.reg_set.set_flag(Flag::Carry); //Rotate right
            } //Rotate right
            self.reg_set.change_single_register(arg_reg, &|x| x.rotate_right(1));
        }
 
        fn rlc(&mut self, arg:Args) -> bool{
            let arg_reg = match arg.get_op(){
                Operand::SingleRegArg(reg) => reg,
                _ => unreachable!()
            };
            if self.reg_set.get_bit(arg_reg, 7){
                self.reg_set.set_flag(Flag::Carry,); //Rotate right
            }
            self.reg_set.change_single_register(arg_reg, &|x| x.rotate_left(1));
        }
        fn rrca(&mut self,arg:Args){
            self.rrc(RegOperand::SingleRegArg(registers::SingleReg::A));
        }
        fn rlca(&mut self, arg:Args){
            self.rlc(Operand::SingleRegArg(registers::SingleReg::A));
        }
        fn rla(&mut self, arg:Args){ //Rotate register a to the left _through_ the carry bti .
            self.rl(Operand::SingleRegArg(registers::SingleReg::A));
        }
        fn rra(&mut self,arg:Args){
            self.rr(make_arg(Operand::SingleRegArg(registers::SingleReg::A)));
        }

        fn daa(&mut self, arg:Args){
            let subtract = self.reg_set.get_flag(registers::Flag::Neg);
            let hcarry = self.reg_set.get_flag(registers::Flag::HalfCarry);
            let carry = self.reg_set.get_flag(registers::Flag::Carry);
            
            //To complete
        }
        fn str_r16_imm(&mut self, arg:Args){ //Properly LD r16 imm16
            
            let reg_pair:DoubleReg = match arg.get_op(){
                Operand::DoubleRegArg(x)=>x,
                _=> unreachable!(),  
            };
            let imm:u16 = match arg.get_imm(){
                Imm::Imm16(x) => x,
                Imm::Imm8(_) => unreachable!()
            };
            self.reg_set.set_double_register(reg_pair,imm2);
            //This may actually also be like... just run str r8 imm twice.
        }
        fn str_r8_imm(&mut self, arg:Args){
            let arg_reg: SingleReg = match arg.get_op(){
                Operand::SingleRegArg(reg) => reg,
                _ => unreachable!()
            };
            let imm:u8 = match arg.get_imm(){
                Imm::Imm8(x) => x,
                Imm::Imm16(_) => unreachable!()
            };
            self.reg_set.set_single_register(arg_reg,imm);
        }
        fn str_r16_acc(&mut self, arg:Args){
            let arg_reg:DoubleReg = match arg.get_op(){
                Operand::DoubleRegArg(reg) => reg,
                _ => unreachable!()
            };
            self.memory_ref.set_memory_16(self.reg_set.get_double_register(arg_reg),self.reg_set.get_acc());
        }
        fn str_c(&mut self, arg:Args){ //Store A at address $FF00+C 

        } 
        fn str_imm8(&mut self, arg:Args){

        } 
        fn str_imm16(&mut self, arg:Args){

        }
        fn ld_imm_sp(&mut self,arg:Args){
            let imm:u16 = match arg.get_imm(){
                Imm::Imm16(x) => x, 
                _ => unreachable!()
            };
            self.reg_set.set_double_register(registers::DoubleReg::SP,imm);
            self.reg_set.increment_pc(1);
        }
        fn ld_r8_r8(&mut self, arg:Args){
            let (reg_dest, reg_src) = match arg.get_op(){
                Operand::PairSingleReg( reg1,reg2) => (reg1,reg2),
                _ => unreachable!()
            };
            self.reg_set.set_single_register(reg_dest,self.reg_set.get_register(reg_src));
            self.waiting = matches!(reg_src,SingleReg::Memptr) || !matches!(reg_dest,SingleReg::Memptr)
        }
        fn ld_acc_addr(&mut self, arg:Args){ //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg:DoubleReg = match arg.get_op(){
                Some(Operand::DoubleReg( reg1,reg2)) => (reg1,reg2),
                _ => unreachable!()
            };
            self.reg_set.set_acc(self.memory_ref.grab_memory_8(self.reg_set.get_double_register(reg)));
            if matches!(reg,DoubleReg::HLP) || matches!(reg,DoubleReg::HLM) {
                self.reg_set.set_double_register(reg,0);
            }
        }
        fn ldh_c(&mut self, arg:Args){ //A = mem($FF00 + c)
            
        } 
        fn ldh_imm8(&mut self, arg:Args){
        }
        fn ldh_imm16(&mut self, arg:Args){
        }
        fn ld_hl_imm8(&mut self, arg:Args){
        }
        fn ld_sp_hl(&mut self, arg:Args){
        }

        fn inc_r8(&mut self, arg:Args){
            let reg:SingleReg = match arg.get_op(){
                Operand::SingleRegArg(rg)=>rg,
                _ => unreachable!()
            }; //self.reg_set.r8_op_mid(self.current_command);
            self.reg_set.change_single_register(reg, &|x| x+1);
            self.waiting = matches!(reg,SingleReg::Memptr)
        }
        fn dec_r8(&mut self, arg:Args){
            let reg:SingleReg = self.reg_set.r8_op_mid(self.current_command);
            self.reg_set.change_single_register(reg, &|x| x-1);
            self.waiting = matches!(reg,SingleReg::Memptr)
        }
        fn inc_r16(&mut self, arg:Args){
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.change_double_register(reg_pair,&|x| x+1);
        }
        fn dec_r16(&mut self, arg:Args){
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            self.reg_set.change_double_register(reg_pair,&|x| x-1);
        }


        fn add_hl(&mut self, arg:Args){
            let reg_pair:DoubleReg = self.reg_set.r16_op(self.current_command);
            let double_reg_val:u16 = self.reg_set.get_double_register(reg_pair);
            self.reg_set.change_double_register(DoubleReg::HL, &|x|x+double_reg_val);
            //TODO: Flags
        }
        fn cpl(&mut self, arg:Args){ //Invert A
            self.reg_set.change_single_register(SingleReg::A,&|x| !x);
            self.reg_set.set_flag(Flag::HalfCarry);
            self.reg_set.set_flag(Flag::Neg);
        }
        fn ccf(&mut self, arg:Args){
            self.reg_set.flip_carry();
        }
        fn scf(&mut self, arg:Args){
            self.reg_set.set_flag(Flag::Carry);
        }
        fn add(&mut self, arg:Args){ 
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };     
            self.reg_set.apply_fun_to_acc( &|x|x+operand)
        } 
        fn adc(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };     
            let carry = self.reg_set.get_flag(Flag::Carry);
            self.reg_set.apply_fun_to_acc( &|x|x+carry+operand)
        } 
        fn sub(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };     
            self.reg_set.apply_fun_to_acc( &|x|x-operand)
        }
        fn subc(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };        
            let carry = self.reg_set.get_flag(Flag::Carry);
            self.reg_set.apply_fun_to_acc(&|x|x-carry-operand);
        } 
        fn and(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };     
            self.reg_set.apply_fun_to_acc(&|x|x&operand);
        } 
        fn xor(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };     
            self.reg_set.apply_fun_to_acc( &|x|x^operand);
        } 
        fn or(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };     
            self.reg_set.apply_fun_to_acc( &|x| x|operand); //what if I were to go even cooler
        }
        fn cp(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Some(Operand::SingleRegArg(x)) => self.reg_set.get_register(x),
                None() => arg.get_imm()?,
                _ => unreachable!()
            };                      
            let acc = self.reg_set.get_register(SingleReg::A);
            self.reg_set.set_flags((arg_reg-acc == 0),true,true,(arg_reg>a)) //TODO: figure our half carry flag.
            //self.reg_set.apply_fun_to_acc(arg_reg, &|x|x&self.reg_set.get_register(process_single_arg(Args)))
        } 
        fn ret_cond(&mut self, arg:Args){

        } 
        fn ret(&mut self, arg:Args){

        }
        fn reti(&mut self, arg:Args){
        } 
        //Jumps
        fn jr_imm(&mut self, arg:Args){
            let next_value:i8 = match arg.get_imm(){
                Some(Imm::Imm8(x)) => x as i8,
                _ => unreachable!()
            };
            //let next_value:i8 = (self.grab_next() as i8);
            self.reg_set.change_double_register(registers::DoubleReg::PC, &|x| x.wrapping_add_signed(next_value.into()));
        }
        fn jr_cond(&mut self, arg:Args){
            if !self.reg_set.get_cond(self.current_command){
                false
            }
            else{
                self.jr_imm(Args)
            }
        }
        fn jp_cond_imm(&mut self, arg:Args){
        
        } 
        fn jp_imm(&mut self, arg:Args){
        
        } 
        fn jp_hl(&mut self, arg:Args){
        
        } 
        fn call_cond(&mut self, arg:Args){
        
        } 
        fn call_imm(&mut self, arg:Args){
        
        } 
        fn rst(&mut self, arg:Args){
        
        }
        fn pop(&mut self, arg:Args){
            let operand:DoubleReg = match arg.get_op(){
                Some(Operand::DoubleRegArg(x)) => x,
                _ => unreachable!()
            };   
            let instruction = self.memory_ref.grab_memory_16(self.reg_set.get_double_register(DoubleReg::SP));
            self.reg_set.set_double_register(operand, instruction);
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x+2)
        
        }
        fn push(&mut self, arg:Args){
            let operand:u16 = match arg.get_op(){
                Some(Operand::DoubleRegArg(x)) => self.reg_set.get_double_register(x),
                _ => unreachable!()
            };   
            self.memory_ref.set_memory_16(self.reg_set.get_double_register(DoubleReg::SP), operand);
            self.reg_set.change_double_register(DoubleReg::SP, &|x|x-2)
        }
        fn add_sp_imm8(&mut self, arg:Args){
            let operand:i8 = match arg.get_imm(){       //Remember, imm8  is a signed value!
                Some(Imm::Imm8(x)) => x as i8,
                _ => unreachable!()
            };
            self.reg_set.change_double_register(registers::DoubleReg::SP, &|x| x.wrapping_add_signed(operand));
 
        }

        fn di(&mut self, arg:Args){
            self.interrupt =match self.interrupt{
                InterruptState::AlmostDisabled|InterruptState::Disabled => InterruptState::Disabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::AlmostDisabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        fn ei(&mut self, arg:Args){
            self.interrupt =match self.interrupt{
                InterruptState::AlmostDisabled|InterruptState::Disabled => InterruptState::AlmostEnabled,
                InterruptState::AlmostEnabled|InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        } 

        fn sla(&mut self, arg:Args){
            self.reg_set.reset_flags();
            let operand:u8 = match arg.get_op(){
                Arg::SingleRegArg(x) => self.reg_set.get_register(x),
                _ => unreachable!()
            };                      
            if operand>127{
                self.reg_set.set_flag(Flag::Carry);
            };
            if operand==128 || operand == 0 {
                self.reg_set.set_flag(Flag::Zero);
            }
            self.reg_set.change_single_register(arg, &|x|x<<1);
        } 
        fn sra(&mut self, arg:Args){
            self.reg_set.reset_flags();
            let operand:u8 = match arg.get_op(){
                Arg::SingleRegArg(x) => self.reg_set.get_register(x),
                _ => unreachable!()
            };                      
            if operand%2 == 1{
                self.reg_set.set_flag(Flag::Carry);
            };
            if operand==1 || operand == 0 {
                self.reg_set.set_flag(Flag::Zero);
            }
            self.reg_set.change_single_register(arg, &|x|(x>>1)+(128*(x>127))); //Sneaky little arithmetic right shift.
        } 
        fn srl(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Arg::SingleRegArg(x) => self.reg_set.get_register(x),
                _ => unreachable!()
            };                      
            if operand%2 == 1{
                self.reg_set.set_flag(Flag::Carry);
            };
            if operand==1 || operand == 0 {
                self.reg_set.set_flag(Flag::Zero);
            }
            self.reg_set.change_single_register(arg, &|x|(x>>1)); 
        } 
        fn swap_r8(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Arg::SingleRegArg(x) => self.reg_set.get_register(x),
                _ => unreachable!()
            };                      
            if operand == 0 {
                self.reg_set.set_flag(Flag::Zero);
            }
             // What exactly is swap? 7 swaps with 3, 6 with 2, 5 with 1, 4 with 0. 
            self.reg_set.change_single_register(arg, &|x:u8|x.rotate_left(4)); 
        } 

        fn bit(&mut self, arg:Args){
            let operand:(u8,u8) = match arg.get_op(){
                Arg::BitSingleReg(x,y) => (x,self.reg_set.get_register(y)),
                _ => unreachable!()
            };  
            self.reg_set.unset_flag(Flag::Neg);
            self.reg_set.unset_flag(Flag::HalfCarry);
            self.reg_set.flag_cond(Flag::Carry,y>>x % 2 == 1);
        }
        fn res(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Arg::BitSingleReg(x,y) => (x,self.reg_set.get_register(y)),
                _ => unreachable!()
            };                      
            self.reg_set.change_single_register(arg, &|x| !(x & 1<<y));
            
        } 
        fn set(&mut self, arg:Args){
            let operand:u8 = match arg.get_op(){
                Operand::BitSingleReg(x,y) => (x,self.reg_set.get_register(y)),
                _ => unreachable!()
            };                      
            self.reg_set.change_single_register(arg, &|x| x | 1<<y );
        }
        fn stop(&mut self, arg:Args){
            loop {
                self.wait(1);
                //PAUSE THE GPU
                //BREAK IF BUTTON PRESSED.
            }
        }
        fn halt(&mut self, arg:Args){
            loop { 
                CpuStruct::wait( 1) //Enter low power mode until an interrupt
            }
        }
        fn cb_block(&mut self, arg:Args ){
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

            //grab the next piece of memory, but we use the CB table.
        }
        fn wait(cycles:u8){
            //4.19 mhz * 4 t cycles 
            thread::sleep(4*CLOCK_PERIOD*cycles.into());
        }
    }
}