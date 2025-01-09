mod cpu_state;
mod function_table;
mod registers;
mod test;

pub mod interrupt;
pub mod cpu {

    use super::cpu_state::cpu_state::CpuState;
    use super::function_table::function_table::FunFind;
    use super::interrupt::interrupt::Interrupt;
    use crate::audio::audio_controller::AudioController;
    use crate::cpu::registers::registers::{DoubleReg, Flag, SingleReg};
    use crate::joypad::joypad::Joypad;
    use log::info;
    use sdl2::render::Canvas;
    use sdl2::video::Window;
    use std::cell::RefCell;
    use std::fs::File;
    use std::ops::Sub;
    use std::rc::Rc;
    use std::thread;
    use std::time::Duration;

    //const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);
    pub type CPUFunct = fn(&mut CpuStruct);
    enum InterruptState {
        Enabled, //Despite naming, it's really that we have E, DI, AD as "enabled" states
        DisableInterrupt,
        //AlmostDisabled, //As it turns out, DI is instant.
        EnableInterrupt,
        AlmostEnabled,
        Disabled,
    }
    pub struct CpuStruct {
        pub cpu_state: CpuState,
        function_lookup: [FunFind; 63],
        cb_block_lookup: [FunFind; 11],
        instruction_register: u8,
        extra_waiting: bool,
        ime_flag: InterruptState,
        stopped: bool,
        clock_cycle_wait: Rc<RefCell<u8>>,
        cb_flag: bool,
        //fetched_instruction:CPUFunct,
        halted: bool, //Preprocess Option<Operand>
                      //used for mem reads to HL, failed conditional jumps
                      //argument:Argument;
    }
    impl CpuStruct {
        pub fn new(
            joypad: Rc<RefCell<Joypad>>,
            audio: AudioController,
            canvas: Canvas<Window>,
            cartridge: File,
        ) -> Self {
            let wait = Rc::new(RefCell::new(0));
            Self {
                cpu_state: CpuState::new(joypad, audio, wait.clone(), canvas, cartridge),
                instruction_register: 0x00, //initalize to a noop
                function_lookup: FunFind::function_lookup(),
                cb_block_lookup: FunFind::cb_block_lookup(),
                extra_waiting: false,
                ime_flag: InterruptState::DisableInterrupt, //Interreupt master enable
                stopped: false,
                halted: false,
                cb_flag: false,
                //fetched_instruction:CpuStruct::nop,
                clock_cycle_wait: wait,
            } //Find a different way of doing this:
              //Break things apart according to our old pipeline model
        }
        pub fn interpret_command(&mut self) {
            //function_lookup:&[FunFind;63], cb_lookup:&[FunFind;11]
            {
                let mut current_wait = self.clock_cycle_wait.borrow_mut();
                if current_wait.gt(&0) {
                    //println!("We decrement wait by one");
                    *current_wait = current_wait.sub(1);
                }
            }
            if self.clock_cycle_wait.borrow().eq(&0) {

                if !self.stopped {
                    //info!("{:?}", self.cpu_state.registers);
                    //println!("We interpret a command");
                    //TODO: Fetch is actually the last part of the instruction, so the PC counter is always one ahead of the actual instruction
                    let current_pc = self.cpu_state.get_pc();
                    //if current_pc>0x90{
                    //    thread::sleep(Duration::new(1, 0))
                    //}
                    self.instruction_register = self.cpu_state.get_byte(current_pc);
                    ////info!("IR:{:X?}", self.instruction_register);
                    let mut taken: bool = false;
                    self.extra_waiting = false;
                    if self.instruction_register != 0xF3 {
                        //0xf3= disable interrutpts
                        self.handle_interrupts();
                    }
                    let mut fun_pointer: fn(&mut CpuStruct) = CpuStruct::nop;
                    let mut waiting = 0;
                    let mut cond_waiting = 0;
                    if !self.cb_flag {
                        for fun_entry in &self.function_lookup {
                            if (self.instruction_register & fun_entry.mask) == fun_entry.value {
                                fun_pointer = fun_entry.function;
                                waiting = fun_entry.wait;
                                if fun_entry.wait_cond.is_some() {
                                    cond_waiting = fun_entry.wait_cond.unwrap();
                                }
                                taken = true;
                            }
                        }
                    } else {
                        self.cb_flag = false;
                        for fun_entry in &self.cb_block_lookup {
                            if (self.instruction_register & fun_entry.mask) == fun_entry.value {
                                fun_pointer = fun_entry.function;
                                waiting = fun_entry.wait;
                                if fun_entry.wait_cond.is_some() {
                                    cond_waiting = fun_entry.wait_cond.unwrap();
                                }
                                taken = true;
                            }
                        }
                    }
                    if !taken {
                        panic!("we didn't do anything!")
                    }
                    fun_pointer(self);
                    if self.extra_waiting {
                        self.wait(waiting);
                    } else {
                        self.wait(cond_waiting);
                    }; //Evaluate for sanity
 //This always overwrites the next instruction.
                } else   {
                    self.handle_interrupts(); //We must handle interrutps for stop case.
                }
                self.cpu_state.inc_pc();
                
            }
            self.cpu_state.on_clock()
            //self.interpret_command(function_lookup, cb_lookup)
            //Manage interrupts
        }
        pub fn cb_block(&mut self) {
            self.cb_flag = true;
        }
        pub fn wait(&mut self, cycles: u8) {
            //We need a way to model this such that we prefix our waits instead of postfixing them.
            *self.clock_cycle_wait.borrow_mut() += cycles;
            //4.19 mhz * 4 t cycles
            //thread::sleep(4 * CLOCK_PERIOD * cycles.into());
        }
        pub fn test_init(&mut self){
            self.cpu_state.set_r8(SingleReg::A, 0x01);
            self.cpu_state.set_flags(true, false, false, false);
            self.cpu_state.set_r8(SingleReg::C, 0x13);
            self.cpu_state.set_r8(SingleReg::E, 0xd8);
            self.cpu_state.set_r8(SingleReg::H, 0x01);
            self.cpu_state.set_r8(SingleReg::L, 0x4D);
            self.cpu_state.set_r16_val(DoubleReg::PC, 0x0100);
            self.cpu_state.set_r16_val(DoubleReg::PC, 0xFFFE);
        }
        pub fn handle_interrupts(&mut self) {
            self.ime_flag = match self.ime_flag {
                //Handle the transition of the IME flag.
                InterruptState::Enabled | InterruptState::AlmostEnabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => InterruptState::AlmostEnabled,
                InterruptState::DisableInterrupt | InterruptState::Disabled => {
                    InterruptState::Disabled
                }
            };
            let are_interreupts_enabled: bool = match self.ime_flag {
                InterruptState::Enabled | InterruptState::DisableInterrupt => true,
                _ => return,
            };
            let enabled_interrupts_flag: u8 = self.cpu_state.get_byte(0xFFFF);
            let mut bit_idx: u8 = 1;
            let mut target_call: u16 = 0x0040;
            let interrupt_flag: u8 = self.cpu_state.get_byte(0xFF0F);
            loop {
                if (interrupt_flag & enabled_interrupts_flag & bit_idx) > 0 {
                    //We process an interrupt
                    if are_interreupts_enabled {
                        //We check in here so that if we have the case where we're halted, and don't have IME Enabled, we unhalt
                        self.cpu_state.set_byte(0xFF0F, interrupt_flag ^ bit_idx); //unset bit
                        self.ime_flag = InterruptState::Disabled;
                        let pc = self.cpu_state.get_r16_val(DoubleReg::PC); //CALL
                        self.cpu_state.set_r16_memory(DoubleReg::SP, pc);
                        self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
                        self.cpu_state.set_pc(target_call);
                        //self.wait(5);
                    }
                    self.halted = false; //Implement the Halt bug!
                    break;
                }
                bit_idx *= 2;
                target_call += 8;
                if bit_idx == 0x20 {
                    break;
                }
            }
            //Vblank has the highest priority
        }
        pub fn cond(&mut self) -> bool {
            self.cpu_state.get_cond(self.instruction_register)
        }
        pub fn alu_register(&mut self) -> SingleReg {
            let ret = match self.instruction_register > 0b11000000 {
                true => SingleReg::A,
                false => self.cpu_state.get_r8_end(self.instruction_register),
            };
            ////info!("ALU Register is:{:?}", ret);
            ret
        }
        pub fn alu_operand(&mut self) -> u8 {
            let ret: u8 = match self.instruction_register > 0xC0 {
                true => self.cpu_state.get_imm8(),
                false => {
                    self.extra_waiting = true;
                    let register = self.cpu_state.get_r8_end(self.instruction_register);
                    self.cpu_state.get_r8_val(register)
                }
            };
            ////info!("ALU register output {:?}", ret);
            ret
        }
        pub fn nop(&mut self) {
            //info!("CPU OP: NOP");
            ()
        }
        // Rotations
        pub fn rl(&mut self) {
            //info!("CPU Operation is Rotate Left through Carry");
            let val = self.alu_operand();
            let reg = self.alu_register();
            let carry: bool = self.cpu_state.get_flag(Flag::Carry);
            self.cpu_state.change_r8(reg, &|x| x << 1 + (carry as u8)); //Check to see if I need wrapping left shift
            self.cpu_state.set_flag(Flag::Carry, val > 127);
        }
        pub fn rlc(&mut self) {
            //Rotate left

            //info!("CPU Operation is Rotate Left, copy Carry");
            let val = self.alu_operand();
            let reg = self.alu_register();
            self.cpu_state.set_flag(Flag::Carry, val > 127);
            self.cpu_state.change_r8(reg, &|x| x.rotate_left(1));
        }
        pub fn rr(&mut self) {
            //info!("CPU Operation is Rotate Right");
            let reg = self.alu_register();
            let val = self.alu_operand();
            let carry: bool = self.cpu_state.get_flag(Flag::Carry);
            let bottom: bool = (val % 2) == 1;
            self.cpu_state.set_flag(Flag::Carry, bottom);
            self.cpu_state
                .change_r8(reg, &|x| (x >> 1) + ((carry as u8) << 7));
        }
        pub fn rrc(&mut self) {
            //info!("CPU Operation is Rotate Right through carry");
            let value = self.alu_operand();
            let reg = self.alu_register();
            self.cpu_state.set_flag(Flag::Carry, (value % 2) == 1);
            self.cpu_state.change_r8(reg, &|x| x.rotate_right(1));
        }
        pub fn daa(&mut self) {
            //info!("CPU Operation is Decimal Adjust Accumulator");
            let subtract = self.cpu_state.get_flag(Flag::Neg);
            let hcarry = self.cpu_state.get_flag(Flag::HalfCarry);
            let carry = self.cpu_state.get_flag(Flag::Carry);
            let mut offset: u8 = 0;
            let a_val = self.cpu_state.get_acc();
            if (!subtract && a_val & 0xf > 0x9) || hcarry {
                offset |= 0x06;
            }
            if (!subtract && a_val > 0x99) || carry {
                offset |= 0x60;
            }
            //let fun = &|x|x+offset;
            if subtract {
                self.cpu_state.apply_fun_to_acc(&|x| x.wrapping_sub(offset));
            } else {
                self.cpu_state.apply_fun_to_acc(&|x| x.wrapping_add(offset));
            }
            self.cpu_state.set_flag(Flag::HalfCarry, false);
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc == 0);
            self.cpu_state.set_flag(Flag::Carry, acc > 0x99);
        }
        //Load Immediate
        pub fn ldi_r16(&mut self) {
            //info!("CPU Operation is Load: MM16->2REG");
            //0x01
            let reg_pair: DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            let imm2: u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(reg_pair, imm2);
            //This may actually also be like... just run str r8 imm twice.
        }
        pub fn ldi_r8(&mut self) {
            //info!("CPU OP: Load IMM8->REG");
            let arg_reg: SingleReg = self.cpu_state.get_r8_mid(self.instruction_register);
            let imm: u8 = self.cpu_state.get_imm8();
            self.cpu_state.set_r8(arg_reg, imm);
        }
        //Stores
        pub fn str_acc_rmem(&mut self) {
            //0x02

            //info!("CPU OP: Store ACC->Mem");
            let arg_reg: DoubleReg = self.cpu_state.r16_mem_tbl(self.instruction_register);
            let mem_addr: u16 = self.cpu_state.get_r16_val(arg_reg);
            let acc_val = self.cpu_state.get_acc();
            self.cpu_state.set_byte(mem_addr, acc_val);
        }
        pub fn str_c(&mut self) {
            //info!("CPU OP: Store ACC->[C+0xFF00]");
            //Store A at address $FF00+C , e2
            let value: u16 = self.cpu_state.get_r8_val(SingleReg::C) as u16 + 0xFF00;
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_byte(value, acc);
        }
        pub fn str_imm8(&mut self) {
            //info!("CPU OP: Store ACC->[IMM8+0xFF00]");
            let reg: u8 = self.cpu_state.get_acc();
            let imm: u16 = self.cpu_state.get_imm8() as u16;
            self.cpu_state.set_byte(imm + 0xFF00, reg);
        }
        pub fn str_imm16(&mut self) {
            //EA
            //info!("CPU OP: Store ACC->[IMM16]");
            let reg: u8 = self.cpu_state.get_acc();
            let imm: u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_byte(imm, reg);
        }
        //Loads
        pub fn ld_imm8(&mut self) {
            //info!("CPU OP: Load: 0xFF00+[IM8]->ACC");
            let imm: u16 = (self.cpu_state.get_imm8() as u16) + 0xFF00;
            let mem: u8 = self.cpu_state.get_byte(imm);
            self.cpu_state.set_acc(mem)
        }
        pub fn ld_imm16(&mut self) {
            //info!("Operation is Load: IMM16->ACC");
            let imm: u16 = self.cpu_state.get_imm16();
            let mem: u8 = self.cpu_state.get_byte(imm);
            self.cpu_state.set_acc(mem)
        }
        pub fn ld_imm_sp(&mut self) {
            //info!("Operation is Load: IMM16->SP");
            let imm: u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(DoubleReg::SP, imm);
        }
        pub fn ld_r8_r8(&mut self) {
            let reg_dest = self.cpu_state.get_r8_mid(self.instruction_register);
            let reg_src = self.cpu_state.get_r8_end(self.instruction_register);

            //info!("CPU OP: Load {:?}->{:?}", reg_src, reg_dest);
            let reg_val = self.cpu_state.get_r8_val(reg_src);
            self.cpu_state.set_r8(reg_dest, reg_val);
            self.extra_waiting = !matches!(reg_dest, SingleReg::Memptr)
        }
        pub fn ld_acc_addr(&mut self) {

            //info!("CPU OP: Load acc->addr");
            //Load from address into accumulator. Kinda similar to LD 7 6 0101110110
            let reg = self.cpu_state.r16_tbl(self.instruction_register);
            let mem: u8 = self.cpu_state.get_r16_memory(reg);
            self.cpu_state.set_acc(mem);
            if matches!(reg, DoubleReg::HLP) || matches!(reg, DoubleReg::HLM) {
                self.cpu_state.set_r16_val(reg, 0);
            }
        }
        pub fn ld_c(&mut self) { 
            //A = mem($FF00 + c)
            //info!("CPU OP: Load High[C]->addr");
            let addr: u16 = self.cpu_state.get_r8_val(SingleReg::C) as u16 + 0xFF00;
            let value: u8 = self.cpu_state.get_byte(addr);
            self.cpu_state.set_acc(value);
        }
        pub fn ld_hl_imm8(&mut self) {
            //f8

            //info!("CPU OP:LOAD SP+IMM -> HL");
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let imm = self.cpu_state.get_imm8() as u16;
            self.cpu_state
                .set_r16_val(DoubleReg::HL, stack_pointer + imm);
        }
        pub fn ld_sp_hl(&mut self) {
            //f9

            //info!("CPU OP:Load stack pointer into HL");
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            self.cpu_state.set_r16_val(DoubleReg::HL, stack_pointer);
        }
        pub fn inc_r8(&mut self) {
            //info!("CPU OP:INC R8");
            let reg: SingleReg = self.cpu_state.get_r8_mid(self.instruction_register); //self.cpu_state.r8_op_mid(self.instruction_register);
            let val: u8 = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, val == 0xff);
            self.cpu_state.set_flag(Flag::HalfCarry, val == 0x0f);
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.change_r8(reg, &|x| x.wrapping_add(1));

            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        pub fn dec_r8(&mut self) {

            //info!("CPU OP:DEC R8");
            let reg: SingleReg = self.cpu_state.get_r8_mid(self.instruction_register);
            let val: u8 = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, val == 0x01);
            self.cpu_state.set_flag(Flag::HalfCarry, val == 0x10);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state.change_r8(reg, &|x| x.wrapping_sub(1));
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        pub fn inc_r16(&mut self) {
            //Doesn't affect flags
            //info!("CPU OP:Increment an R16");
            let reg_pair: DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            self.cpu_state.change_r16(reg_pair, &|x| x + 1);
        }
        pub fn dec_r16(&mut self) {
            //info!("CPU OP:Decrement an R16");
            let reg_pair: DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            self.cpu_state.change_r16(reg_pair, &|x| x - 1);
        }
        pub fn add_hl(&mut self) {
            //info!("CPU OP:Increment HL");
            let reg_pair: DoubleReg = self.cpu_state.r16_tbl(self.instruction_register);
            let operand: u16 = self.cpu_state.get_r16_val(reg_pair);
            let hl_val: u16 = self.cpu_state.get_r16_val(DoubleReg::HL);
            let result = self
                .cpu_state
                .change_r16(DoubleReg::HL, &|x| x.wrapping_add(operand));
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.set_flag(
                Flag::HalfCarry,
                (hl_val & 0x0fff) + (operand & 0x0fff) > 0x1000,
            );
            self.cpu_state
                .set_flag(Flag::Carry, None == hl_val.checked_add(operand));
            self.cpu_state.set_flag(Flag::Zero, result == 0);
        }
        pub fn cpl(&mut self) {
            //Invert A

            //info!("CPU OP: Invert A");
            self.cpu_state.set_flag(Flag::HalfCarry, true);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state.change_r8(SingleReg::A, &|x| !x);
        }
        pub fn ccf(&mut self) {
            //info!("CPU OP: Flip Carry Flag");
            self.cpu_state.flip_carry();
        }
        pub fn scf(&mut self) {
            //info!("CPU OP: Set Carry Flag");
            self.cpu_state.set_flag(Flag::Carry, true);
        }
        pub fn add(&mut self) {
            //info!("CPU OP: Add without carry");
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            let ret = self
                .cpu_state
                .apply_fun_to_acc(&|x| x.wrapping_add(operand));
            self.cpu_state.set_flag(Flag::Zero, ret == 0);
            self.cpu_state
                .set_flag(Flag::Carry, None == acc.checked_add(operand));
            self.cpu_state
                .set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F)) > 0x10);
            self.cpu_state.set_flag(Flag::Neg, false);
        }
        pub fn adc(&mut self) {
            //info!("CPU OP: Add with carry");
            let carry = self.cpu_state.get_flag(Flag::Carry);
            let operand = self.alu_operand() + (carry as u8);
            let acc = self.cpu_state.get_acc();
            self.cpu_state
                .set_flag(Flag::Zero, acc.wrapping_add(operand) == 0);
            self.cpu_state
                .set_flag(Flag::Carry, None == acc.checked_add(operand));
            self.cpu_state.set_flag(
                Flag::HalfCarry,
                ((acc & 0x0F) + (operand & 0x0F) as u8) > 0x10,
            );
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state
                .apply_fun_to_acc(&|x| x.wrapping_add(operand));
        }
        pub fn sub(&mut self) {
            //info!("CPU OP: Sub without carry");
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc == operand);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state
                .set_flag(Flag::HalfCarry, (acc & 0x0F) < (operand & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand > acc);
            self.cpu_state
                .apply_fun_to_acc(&|x| x.wrapping_sub(operand));
        }
        pub fn subc(&mut self) {
            //info!("CPU OP: Sub with Carry");
            let carry: u8 = self.cpu_state.get_flag(Flag::Carry) as u8;
            let operand: u8 = self.alu_operand() + carry;
            let acc: u8 = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc == operand);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state
                .set_flag(Flag::HalfCarry, (acc & 0x0F) < ((operand) & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand > acc);
            self.cpu_state
                .apply_fun_to_acc(&|x| x.wrapping_sub(operand));
        }
        pub fn and(&mut self) {
            //info!("CPU OP: And");
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state
                .set_flags(operand & acc == 0, false, true, false);
            self.cpu_state.apply_fun_to_acc(&|x| x & operand);
        }
        pub fn xor(&mut self) {
            //info!("CPU OP: Xor");
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state
                .set_flags(operand ^ acc == 0, false, true, false);
            self.cpu_state.apply_fun_to_acc(&|x| x ^ operand);
        }
        pub fn or(&mut self) {
            //info!("CPU OP: Or");
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state
                .set_flags(operand | acc == 0, false, true, false);
            self.cpu_state.apply_fun_to_acc(&|x| x | operand); //what if I were to go even cooler
        }
        pub fn cp(&mut self) {
            //info!("CPU OP: Compare");
            let operand = self.alu_operand();
            let acc = self.cpu_state.get_acc();
            self.cpu_state.set_flag(Flag::Zero, acc == operand);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state
                .set_flag(Flag::HalfCarry, (acc & 0x0F) < (operand & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand > acc);
        }

        pub fn ret(&mut self) {
            //info!("CPU OP: Return");
            let instruction = self.cpu_state.get_r16_memory_word(DoubleReg::SP);
            //info!("Return pointer:{:?}",instruction);
            self.cpu_state.set_r16_val(DoubleReg::PC, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
        }
        pub fn ret_cond(&mut self) {
            //info!("CPU OP: Return conditional");
            if self.cond() {
                let instruction = self.cpu_state.get_r16_memory_word(DoubleReg::SP);
                self.cpu_state.set_r16_val(DoubleReg::PC, instruction);
                self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
                self.extra_waiting = true;
            }
        }
        pub fn reti(&mut self) {
            //info!("CPU OP: Return, Enable Interrupts");
            let instruction = self.cpu_state.get_r16_memory_word(DoubleReg::SP);
            self.cpu_state.set_r16_val(DoubleReg::PC, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
            self.ei();
        }
        //Jumps
        pub fn jr_imm(&mut self) {
            //Jump Relative

            //info!("CPU OP: Jump Relative");
            let imm = self.cpu_state.get_imm8() as i8;
            let pc = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_pc(pc.wrapping_add_signed(imm.into()));
        }
        pub fn jr_cond(&mut self) {
            //info!("CPU OP: Jump Conditional Relative");
            if self.cond() {
                self.jr_imm();
                self.extra_waiting = true;
            }else{
               self.cpu_state.get_imm8();
            }
        }

        pub fn jp_imm(&mut self) {
            //info!("CPU OP: Jump to IMM8");
            let imm: u16 = self.cpu_state.get_imm16();
            self.cpu_state.set_r16_val(DoubleReg::PC, imm);
        }
        pub fn jp_cond_imm(&mut self) {
            //info!("CPU OP: Jump Conditional To IMM8");
            if self.cond() {
                self.jp_imm();
                self.extra_waiting = true;
            }else{
               self.cpu_state.get_imm16();
            }
        }
        pub fn jp_hl(&mut self) {
            //info!("CPU OP:Jump HL");
            let hl: u16 = self.cpu_state.get_r16_val(DoubleReg::HL);
            self.cpu_state.set_r16_val(DoubleReg::PC, hl)
        }
        pub fn call_cond(&mut self) {
            //info!("CPU OP: Cond Call");
            if self.cond() {
                self.call_imm();
                self.extra_waiting = true;
            }else{ 
                self.cpu_state.get_imm16();
            }
        }
        pub fn call_imm(&mut self) {
            //info!("CPU OP: Call IMM");
            //thread::sleep(time::Duration::from_secs(2));

            self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
            let addr = self.cpu_state.get_imm16();
            let pc = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_r16_memory(DoubleReg::SP, pc);
            self.cpu_state.set_pc(addr);
        }
        pub fn rst(&mut self) {
            //info!("OP: RST");
            //dbg!("OP: RST");
            let pc: u16 = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_r16_memory(DoubleReg::SP, pc);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
            self.cpu_state
                .set_pc((self.instruction_register & 0x38) as u16);
        }
        pub fn pop(&mut self) {
            //info!("CPU OP: POP");

            let operand = self.cpu_state.r16_stk_tbl(self.instruction_register);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let instruction = self.cpu_state.get_half_word(stack_pointer);
            //info!("{:X?}",instruction);
            //thread::sleep(Duration::from_secs(5));
            self.cpu_state.set_r16_val(operand, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
        }
        pub fn push(&mut self) {
            //info!("CPU OP: PUSH");
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
            let operand = self.cpu_state.r16_stk_tbl(self.instruction_register);
            let value: u16 = self.cpu_state.get_r16_val(operand);

            //info!("PUSH VAL {:X?}",value);

            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            self.cpu_state.set_half_word(stack_pointer, value);
            //thread::sleep(Duration::from_secs(5));
        }
        pub fn add_sp_imm8(&mut self) {

            //info!("CPU OP: SP=SP+imm8");
            let operand: i8 = self.cpu_state.get_imm8() as i8;
            //self.cpu_state.set_flag(Flag::Carry, None == acc.checked_add(operand) );
            //self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x0F) + (operand & 0x0F)) > 0x10 );
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.set_flag(Flag::Zero, false);
            self.cpu_state
                .change_r16(DoubleReg::SP, &|x| x.wrapping_add_signed(operand as i16));
        }

        pub fn di(&mut self) {
            //info!("CPU OP: Disable Interrupts");
            self.ime_flag = match self.ime_flag {
                InterruptState::Disabled => InterruptState::Disabled,
                InterruptState::AlmostEnabled | InterruptState::Enabled => {
                    InterruptState::DisableInterrupt
                }
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        pub fn ei(&mut self) {
            //info!("CPU OP: Enable Interrupts");
            self.ime_flag = match self.ime_flag {
                InterruptState::Disabled => InterruptState::EnableInterrupt,
                InterruptState::AlmostEnabled | InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        pub fn sla(&mut self) {

            //info!("CPU OP: Shift left into carry");
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state
                .set_flags(operand == 128 || operand == 0, false, false, operand > 127);
            self.cpu_state.change_r8(reg, &|x| x << 1);
        }
        pub fn sra(&mut self) {

            //info!("CPU OP: Shift right into carry");
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state
                .set_flags(operand == 1 || operand == 0, false, false, operand % 2 == 1);
            self.cpu_state
                .change_r8(reg, &|x| (x >> 1) + (128 * ((x > 127) as u8))); //Sneaky little arithmetic right shift.
        }
        pub fn srl(&mut self) {

            //info!("CPU OP: Shift right(logical)");
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state
                .set_flags(operand == 1 || operand == 0, false, false, operand % 2 == 1);
            self.cpu_state.change_r8(reg, &|x| (x >> 1));
        }
        pub fn swap(&mut self) {

            //info!("CPU OP: Swap nibbles");
            let reg = self.cpu_state.get_r8_end(self.instruction_register);
            let operand = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flags(operand == 0, false, false, false);
            self.cpu_state.change_r8(reg, &|x: u8| x.rotate_left(4));
        }
        pub fn bit(&mut self) {
            //info!("CPU OP: CHECK BIT");
            let bits: u8 = (self.instruction_register & 63) >> 3;
            //info!("WE CHECK BIT {:?}", bits);
            let reg: SingleReg = self.cpu_state.get_r8_end(self.instruction_register);
            let val: u8 = self.cpu_state.get_r8_val(reg);
            //info!("ZERO FLAG?{:?}", ((val >> bits) % 2) == 0);
            self.cpu_state
                .set_flag(Flag::Zero, ((val >> bits) % 2) == 0);
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.set_flag(Flag::HalfCarry, true); //???x
        }
        pub fn res(&mut self) {

            //info!("CPU OP: RESET BIT");
            let bits: u8 = (self.instruction_register & 63) >> 3;
            let reg: SingleReg = self.cpu_state.get_r8_end(self.instruction_register);
            self.cpu_state.change_r8(reg, &|x| !(x & (1 << bits)));
        }
        pub fn set(&mut self) {

            //info!("CPU OP: SET BIT");
            let bits: u8 = (self.instruction_register & 63) >> 3;
            let reg: SingleReg = self.cpu_state.get_r8_end(self.instruction_register);
            self.cpu_state.change_r8(reg, &|x| x | 1 << bits);
        }
        pub fn stop(&mut self) {
            //No official rom uses stop, so we're using the stop flag for halt instead
            self.stopped = true;
        }
        pub fn halt(&mut self) {
            //We need to implement the halt bug where we repeat the PC counter.
            self.stopped = true;
            self.ime_flag = InterruptState::Enabled;
        }

        pub fn interrupt(&mut self, interrupt: Interrupt) {
            //info!("Hanle interrupts Interrupts");
            match self.ime_flag {
                InterruptState::Enabled | InterruptState::DisableInterrupt => (),
                _ => return (),
            };
            let bit_idx: u8 = match interrupt {
                Interrupt::VBlank => 0,
                Interrupt::LCDC => 1,
                Interrupt::Timer => 2,
                Interrupt::Serial => 3,
                Interrupt::Input => 4,
            };
            let mut current_interrupt_flag: u8 = self.cpu_state.get_byte(0xFF0F);
            current_interrupt_flag |= 1 << bit_idx;
            self.cpu_state.set_byte(0xFF0F, current_interrupt_flag);
        }
    }
}
