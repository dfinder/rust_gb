mod cpu_state;
mod function_table;
mod registers;
mod test;

pub mod interrupt;
pub mod cpu {

    use super::cpu_state::cpu_state::CpuState;

    use super::function_table::function_table::{CPUFn, Dest16, Dest8, FunFind, Src16, Src8};
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
    use std::thread::{self, Thread};
    use std::time::Duration;
    use std::{mem, u8};
    pub type CPUFunct2<'a> = &'a dyn Fn(&mut CpuStruct);

    //const CLOCK_PERIOD: time::Duration = Duration::from_nanos(239);

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
        function_lookup: [FunFind;63],
        cb_block_lookup: [FunFind;11],
        instruction_register: u8,
        //boot_rom_double: File,
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
            joypad: Joypad,
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
                ime_flag: InterruptState::DisableInterrupt, //Interreupt master enable
                stopped: false,
                halted: false,
                cb_flag: false,
                //boot_rom_double: File::create("BOOT_ROM_DOUBLE.bin").expect("test"),
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

            let mut waiting = 0;
            if self.clock_cycle_wait.borrow().eq(&0) {
                if !self.stopped {
                    let current_pc = self.cpu_state.get_pc();
                    self.instruction_register = self.cpu_state.get_byte(current_pc);
                    info!("R:{:?}", self.cpu_state.registers);
                    //info!("IR:{:X?}", self.instruction_register);
                    if (0x0095..0x00A7).contains(&current_pc ){
                        ()//thread::sleep(Duration::from_secs(1));
                    } 
                    if self.instruction_register != 0xF3 {
                        //0xf3= disable interrutpts
                        self.handle_interrupts();
                    }
                    if !self.cb_flag {
                        for fun_entry in self.function_lookup {
                            if (self.instruction_register & fun_entry.mask) == fun_entry.value {
                                match fun_entry.function{
                                    CPUFn::Ld8(src_enum, dest_enum) => self.ld(&src_enum, &dest_enum),
                                    CPUFn::Ld16(src16, dest16) => self.ld16( &src16, &dest16),
                                    CPUFn::ALU8(alu_func) => {
                                        //Where we're storing things, usually A
                                        
                                        let op_1 = self.cpu_state.get_acc();
                                        let op_2 = self.alu_operand();
                                        let val: u8 = alu_func(self, op_1, op_2);
                                        self.cpu_state.set_acc(val)
                                    },
                                    CPUFn::ALU8Self(alu_func) => {
                                        let reg = self.cpu_state.get_r8_end(self.instruction_register); //Where we're storing things
                                        let op = self.cpu_state.get_r8_val(reg);
                                        let val: u8 = alu_func(self, op);
                                        self.cpu_state.set_r8(reg, val);
                                    },
                                    CPUFn::ALU16Self(alu_fun) => {
                                        let reg: DoubleReg =
                                            self.cpu_state.r16_tbl(self.instruction_register);
                                        let op = self.cpu_state.get_r16_val(reg);
                                        let val = alu_fun(self, op);
                                        self.cpu_state.set_r16(reg, val);
                                    },
                                    CPUFn::Other(x) => x(self),
                                };
                                waiting = fun_entry.wait;
                                break;
                            }
                        }
                    } else {
                        {
                            self.cb_flag = false;
                            let lookup = &self.cb_block_lookup;
                            for fun_entry in lookup {
                                if (self.instruction_register & fun_entry.mask) == fun_entry.value {
                                    match fun_entry.function {
                                        CPUFn::ALU8Self(alu_func) => {
                                            let reg = self.cpu_state.get_r8_end(self.instruction_register); //Where we're storing things
                                            let op = self.cpu_state.get_r8_val(reg);
                                            let val: u8 = alu_func(self, op);
                                            self.cpu_state.set_r8(reg, val);
                                        }
                                        _ => unreachable!(),
                                    };
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    self.handle_interrupts(); //We must handle interrutps for stop case.
                }

                self.cpu_state.inc_pc();
                self.wait(waiting);
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
        pub fn test_init(&mut self) {
            self.cpu_state.set_r8(SingleReg::A, 0x01);
            self.cpu_state.set_flags(true, false, false, false);
            self.cpu_state.set_r8(SingleReg::C, 0x13);
            self.cpu_state.set_r8(SingleReg::E, 0xd8);
            self.cpu_state.set_r8(SingleReg::H, 0x01);
            self.cpu_state.set_r8(SingleReg::L, 0x4D);
            self.cpu_state.set_r16(DoubleReg::PC, 0x0100);
            //self.cpu_state.set_r16_val(DoubleReg::PC, 0xFFFE);
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
                        self.cpu_state.set_r16_mem_16(DoubleReg::SP, pc);
                        self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
                        self.cpu_state.set_pc(target_call);
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
            let ret = match self.instruction_register > 0xc0{
                false => SingleReg::A,
                true => self.cpu_state.get_r8_end(self.instruction_register),
            };

            //info!("instruction register is:{:x?}", self.instruction_register);
            //info!("ALU Register is:{:?}", ret);
            ret
        }
        pub fn alu_operand(&mut self) -> u8 {
            //let ret: u8 =
            match self.instruction_register > 0xC0 {
                true => self.cpu_state.get_imm8(),
                false => self.get_r8_end_val()
            }
            //info!("ALU register output {:?}", ret);
            //ret
        }
        pub fn nop(&mut self) {
            //info!("CPU OP: NOP");
            ()
        }
        // Rotations
        pub fn rl(&mut self, op: u8) -> u8 {
            info!("CPU Operation is Rotate Left through Carry");
            let carry: bool = self.cpu_state.get_flag(Flag::Carry);
            info!("{:X?}",op);
            self.cpu_state.set_flag(Flag::Carry, op > 127);
            (op << 1) + (carry as u8)
        }

        pub fn rlc(&mut self, op: u8) -> u8 {
            //Rotate left
            //info!("CPU Operation is Rotate Left circular");
            self.cpu_state.set_flag(Flag::Carry, op > 127);
            op.rotate_left(1)
        }
        pub fn rr(&mut self, op: u8) -> u8 {
            //info!("CPU Operation is Rotate Right");
            let carry: bool = self.cpu_state.get_flag(Flag::Carry);
            let bottom: bool = (op % 2) == 1;
            self.cpu_state.set_flag(Flag::Carry, bottom);
            (op >> 1) + ((carry as u8) << 7)
        }
        pub fn rrc(&mut self, op: u8) -> u8 {
            //info!("CPU Operation is Rotate Right circular");
            self.cpu_state.set_flag(Flag::Carry, (op % 2) == 1);
            op.rotate_right(1)
        }

        pub fn daa(&mut self, acc: u8) -> u8 {
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
            let ret;
            if subtract {
                ret = acc.wrapping_sub(offset);
            } else {
                ret = acc.wrapping_add(offset);
            }
            self.cpu_state.set_flag(Flag::HalfCarry, false);
            self.cpu_state.set_flag(Flag::Zero, ret == 0);
            self.cpu_state.set_flag(Flag::Carry, ret > 0x99);
            ret
        }
        //Load Immediate

        pub fn ld_hl_imm8(&mut self) {
            //f8
            //info!("CPU OP:LOAD SP+IMM -> HL");
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let imm = self.cpu_state.get_imm8();
            
            self.cpu_state
                .set_r16(DoubleReg::HL, stack_pointer - (0x100 - (imm as u16)));
        }
        pub fn ld16(&mut self, source: &Src16, dest: &Dest16) {
            let value:u16 = match source {
                Src16::Imm16 => {
                    self.cpu_state.get_imm16()

                },
                Src16::HL => self.cpu_state.get_r16_val(DoubleReg::HL),
                Src16::SP => self.cpu_state.get_r16_val(DoubleReg::SP),
            };
            //info!("{:X?}",value);
            match dest {
                Dest16::R16 => {
                    let reg = self.cpu_state.r16_tbl(self.instruction_register);
                    self.cpu_state.set_r16( reg, value);
                }
                Dest16::PC =>  {
                    self.cpu_state.set_r16( DoubleReg::PC, value);
                },
                Dest16::SP =>  {
                    self.cpu_state.set_r16( DoubleReg::SP, value);
                },
                Dest16::HL =>  {
                    self.cpu_state.set_r16( DoubleReg::HL, value);
                },
            }
        }
        pub fn ld(&mut self, source: &Src8, dest: &Dest8) {
            let source_value: u8 = match source {
                Src8::Imm16Mem => {
                    let imm = self.cpu_state.get_imm16();
                    self.cpu_state.get_byte(imm)
                }
                Src8::Imm8 => self.cpu_state.get_imm8(),
                Src8::HighBank => {
                    let imm = self.cpu_state.get_imm8();
                    self.cpu_state.get_byte(u16::from_be_bytes([0xff, imm]))
                }
                Src8::Acc => self.cpu_state.get_acc(),
                
                Src8::R8Mid => {
                    let reg =self.get_r8_mid();

                    self.cpu_state.get_r8_val(reg)
                },
                Src8::R8 => self.get_r8_end_val(),
                Src8::HighC => {
                    let imm = self.cpu_state.get_r8_val(SingleReg::C);
                    self.cpu_state.get_byte(u16::from_be_bytes([0xff, imm]))
                },
                Src8::R16Mem => {
                    let reg = self.cpu_state.r16_mem_tbl(self.instruction_register);
                    self.cpu_state.get_r16_mem_8(reg)
                },
            };
            match dest {
                Dest8::R8 => {
                    let reg = self.get_r8_mid();
                    self.cpu_state.set_r8(reg, source_value)
                }
                Dest8::Imm16Mem => {
                    let dest = self.cpu_state.get_imm16();
                    self.cpu_state.set_byte(dest, source_value);
                }
                Dest8::Imm8High => {
                    let imm = self.cpu_state.get_imm8();
                    
                    //info!("{:X?}",source_value);
                    //info!("{:X?}",u16::from_be_bytes([0xff, imm]));
                    //thread::sleep(Duration::from_secs(1));
                    self.cpu_state
                        .set_byte(u16::from_be_bytes([0xff, imm]), source_value);
                }
                Dest8::Acc => self.cpu_state.set_acc(source_value),
                Dest8::R16Mem => {
                    let double_reg = self.cpu_state.r16_mem_tbl(self.instruction_register);
                    self.cpu_state.set_r16_mem_8(double_reg, source_value);
                }
                Dest8::HighC => {
                    let c_val = self.cpu_state.get_r8_val(SingleReg::C);
                    self.cpu_state
                        .set_byte(u16::from_be_bytes([0xff, c_val]), source_value)
                }
            }
        }
        pub fn get_r8_mid(&mut self) -> SingleReg{
            match (self.instruction_register >> 3) % 8 {
                0 => SingleReg::B,
                1 => SingleReg::C,
                2 => SingleReg::D,
                3 => SingleReg::E,
                4 => SingleReg::H,
                5 => SingleReg::L,
                6 => SingleReg::Memptr,
                7 => SingleReg::A,
                _ => unreachable!(),
            }
        }
        pub fn get_r8_end_val(&mut self) -> u8 {
            let r8 = match self.instruction_register % 8 {
                0 => SingleReg::B,
                1 => SingleReg::C,
                2 => SingleReg::D,
                3 => SingleReg::E,
                4 => SingleReg::H,
                5 => SingleReg::L,
                6 => SingleReg::Memptr,
                7 => SingleReg::A,
                _ => unreachable!(),
            };
            self.cpu_state.get_r8_val(r8)
        }
        pub fn inc_r8(&mut self) {
            //info!("CPU OP:INC R8");
            let reg = self.get_r8_mid();
            let mut op = self.cpu_state.get_r8_val(reg);

            self.cpu_state.set_flag(Flag::Zero, op == 0xff);
            self.cpu_state.set_flag(Flag::HalfCarry, op == 0x0f);
            self.cpu_state.set_flag(Flag::Neg, false);
            op = op.wrapping_add(1);
            self.cpu_state.set_r8(reg, op);
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        pub fn dec_r8(&mut self) {
            //info!("CPU OP:INC R8");
            let reg = self.get_r8_mid();
            let mut op = self.cpu_state.get_r8_val(reg);
            self.cpu_state.set_flag(Flag::Zero, op == 1);
            self.cpu_state.set_flag(Flag::HalfCarry, op == 0x10);
            self.cpu_state.set_flag(Flag::Neg, true);
            op = op.wrapping_sub(1);
            self.cpu_state.set_r8(reg, op);
            //self.extra_waiting = matches!(reg,SingleReg::Memptr)
        }
        pub fn inc_r16(&mut self, op: u16) -> u16 {
            //Doesn't affect flags
            //info!("CPU OP:Increment an R16");
            op.wrapping_add(1)
        }
        pub fn dec_r16(&mut self, op: u16) -> u16 {
            //info!("CPU OP:Decrement an R16");
            op.wrapping_sub(1)
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
        pub fn add(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Add without carry");
            let ret = acc.wrapping_add(op);
            self.cpu_state.set_flag(Flag::Zero, ret == 0);
            self.cpu_state
                .set_flag(Flag::Carry, acc.checked_add(op).is_none());
            self.cpu_state
                .set_flag(Flag::HalfCarry, ((acc & 0x0F) + (op & 0x0F)) > 0x10);
            self.cpu_state.set_flag(Flag::Neg, false);
            ret
        }
        pub fn adc(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Add with carry");
            let carry = self.cpu_state.get_flag(Flag::Carry);
            let operand = op + (carry as u8);
            self.cpu_state
                .set_flag(Flag::Zero, acc.wrapping_add(operand) == 0);
            self.cpu_state
                .set_flag(Flag::Carry, acc.checked_add(operand).is_none());
            self.cpu_state.set_flag(
                Flag::HalfCarry,
                ((acc & 0x0F) + (operand & 0x0F) as u8) > 0x10,
            );
            self.cpu_state.set_flag(Flag::Neg, false);
            acc.wrapping_add(operand)
        }
        pub fn sub(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Sub without carry");
            self.cpu_state.set_flag(Flag::Zero, acc == op);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state
                .set_flag(Flag::HalfCarry, (acc & 0x0F) < (op & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, op > acc);
            //thread::sleep(Duration::from_secs(1));
            acc.wrapping_sub(op)
        }
        pub fn subc(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Sub with Carry");
            let carry: u8 = self.cpu_state.get_flag(Flag::Carry) as u8;
            let operand: u8 = self.alu_operand() + carry;
            self.cpu_state.set_flag(Flag::Zero, acc == operand);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state
                .set_flag(Flag::HalfCarry, (acc & 0x0F) < ((operand) & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, operand > acc);
            acc.wrapping_sub(op)
        }
        pub fn and(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: And");
            self.cpu_state.set_flags(op & acc == 0, false, true, false);
            acc & op
        }
        pub fn xor(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Xor");;
            self.cpu_state
                .set_flags((acc ^ op) == 0, false, true, false);
            acc ^ op
        }
        pub fn or(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Or");
            self.cpu_state.set_flags(acc | op == 0, false, true, false);
            acc | op
        }
        pub fn cp(&mut self, acc: u8, op: u8) -> u8 {
            //OP1 is Accumulate
            //info!("CPU OP: Compare");
            //info!("Operand:{:?}",operand);
            self.cpu_state.set_flag(Flag::Zero, acc == op);
            self.cpu_state.set_flag(Flag::Neg, true);
            self.cpu_state
                .set_flag(Flag::HalfCarry, (acc & 0x0F) < (op & 0x0F));
            self.cpu_state.set_flag(Flag::Carry, op > acc);
            acc
        }
        pub fn sla(&mut self, op: u8) -> u8 {
            //info!("CPU OP: Shift left into carry");
            self.cpu_state
                .set_flags(op == 128 || op == 0, false, false, op > 127);
            op << 1
        }
        pub fn sra(&mut self, op: u8) -> u8 {
            //info!("CPU OP: Shift right into carry");
            self.cpu_state
                .set_flags(op == 1 || op == 0, false, false, op % 2 == 1);
            //Sneaky little arithmetic right shift.
            (op >> 1) + (128 * ((op > 127) as u8))
        }
        pub fn srl(&mut self, op: u8) -> u8 {
            self.cpu_state.set_flags(op < 2, false, false, op % 2 == 1);
            op >> 1
        }
        pub fn swap(&mut self, op: u8) -> u8 {
            //info!("CPU OP: Swap nibbles");
            self.cpu_state.set_flags(op == 0, false, false, false);
            op.rotate_left(4)
        }
        pub fn bit(&mut self, op: u8) -> u8 {
            //info!("CPU OP: CHECK BIT");
            let bits: u8 = (self.instruction_register & 63) >> 3;
            self.cpu_state.set_flag(Flag::Zero, ((op >> bits) % 2) == 0);
            self.cpu_state.set_flag(Flag::Neg, false);
            self.cpu_state.set_flag(Flag::HalfCarry, true); //???x
            op
        }
        pub fn res(&mut self, op: u8) -> u8 {
            //info!("CPU OP: RESET BIT");
            let bits: u8 = (self.instruction_register & 0x3F) >> 3;
            op & !(1 << bits)
        }
        pub fn set(&mut self, op: u8) -> u8 {
            //info!("CPU OP: SET BIT");
            let bits: u8 = (self.instruction_register & 0x3F) >> 3;
            op | (1 << bits)
        }
        pub fn ret(&mut self) {
            //info!("CPU OP: Return");
            let instruction = self.cpu_state.get_r16_mem_16(DoubleReg::SP);
            //info!("Return pointer:{:X?}",instruction);
            self.cpu_state.set_r16(DoubleReg::PC, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
        }
        pub fn ret_cond(&mut self) {
            //info!("CPU OP: Return conditional");
            if self.cond() {
                let instruction = self.cpu_state.get_r16_mem_16(DoubleReg::SP);
                self.cpu_state.set_r16(DoubleReg::PC, instruction);
                self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
                //self.extra_waiting = true;
            }
        }
        pub fn reti(&mut self) {
            //info!("CPU OP: Return, Enable Interrupts");
            let instruction = self.cpu_state.get_r16_mem_16(DoubleReg::SP);
            self.cpu_state.set_r16(DoubleReg::PC, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);

            self.ime_flag = match self.ime_flag {
                InterruptState::Disabled => InterruptState::EnableInterrupt,
                InterruptState::AlmostEnabled | InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        //Jumps
        pub fn jr_imm(&mut self) {
            //Jump Relative

            //info!("CPU OP: Jump Relative");
            let imm = self.cpu_state.get_simm8() as i8;
            let pc = self.cpu_state.get_r16_val(DoubleReg::PC);

          
            self.cpu_state.set_pc(pc.wrapping_add_signed(imm.into()));
        }
        pub fn jr_cond(&mut self) {
            //info!("CPU OP: Jump Conditional Relative");
            if self.cond() {
                self.jr_imm();
            } else {
                self.cpu_state.inc_pc();
            }
        }
        pub fn jp_cond_imm(&mut self) {
            //info!("CPU OP: Jump Conditional To IMM16");
            let imm: u16 = self.cpu_state.get_imm16();
            if self.cond() {
                self.cpu_state.set_pc(imm);
            } 
        }
        pub fn call_cond(&mut self) {
            //info!("CPU OP: Cond Call");
            if self.cond() {
                self.call_imm();
            } else {
                self.cpu_state.inc_pc();
                self.cpu_state.inc_pc();
            }
        }
        pub fn call_imm(&mut self) {
            //info!("CPU OP: Call IMM");
            //thread::sleep(time::Duration::from_secs(2));
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
            let addr = self.cpu_state.get_imm16();
            let pc = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_r16_mem_16(DoubleReg::SP, pc);
            self.cpu_state.set_pc(addr-1); //This fixed a call bug
        }
        pub fn rst(&mut self) {
            //info!("OP: RST");
            //dbg!("OP: RST");
            let pc: u16 = self.cpu_state.get_r16_val(DoubleReg::PC);
            self.cpu_state.set_r16(DoubleReg::SP, pc);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
            self.cpu_state
                .set_pc(((self.instruction_register & 0x38) - 1) as u16);
        }
        pub fn pop(&mut self) {
            //info!("CPU OP: POP");
            let operand = self.cpu_state.r16_stk_tbl(self.instruction_register);
            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            let instruction = self.cpu_state.get_mem_16(stack_pointer);
            //info!("{:X?}",instruction);
            //thread::sleep(Duration::from_secs(5));
            self.cpu_state.set_r16(operand, instruction);
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x + 2);
        }
        pub fn push(&mut self) {
            //info!("CPU OP: PUSH");
            self.cpu_state.change_r16(DoubleReg::SP, &|x| x - 2);
            let operand = self.cpu_state.r16_stk_tbl(self.instruction_register);
            let value: u16 = self.cpu_state.get_r16_val(operand);

            //info!("PUSH VAL {:X?}",value);

            let stack_pointer = self.cpu_state.get_r16_val(DoubleReg::SP);
            self.cpu_state.set_mem_16(stack_pointer, value);
            //thread::sleep(Duration::from_secs(5));
        }
        pub fn add_sp_imm8(&mut self) {
            //info!("CPU OP: SP=SP+imm8");
            let acc = self.cpu_state.get_r16_val(DoubleReg::SP);
            let operand: i8 = self.cpu_state.get_simm8();
            self.cpu_state.set_flag(Flag::Carry, acc.checked_add_signed(operand as i16).is_none() );
            //self.cpu_state.set_flag(Flag::HalfCarry, ((acc & 0x000F).saturating_add(operand & 0x0F)) > 0x10 );
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
        
        pub fn stop(&mut self) {
            //No official rom uses stop, so we're using the stop flag for halt instead
            self.stopped = true;
        }
        pub fn halt(&mut self) {
            //We need to implement the halt bug where we repeat the PC counter.
            self.stopped = true;
            self.ime_flag = InterruptState::Enabled;
        }
    }
}
