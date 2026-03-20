mod function_table;
mod test;

pub mod cpu {

    use super::function_table::function_table::{CPUFn, Dest16, Dest8, FunFind, Src16, Src8};
    use crate::audio::audio_controller::AudioController;
    use crate::joypad::joypad::Joypad;
    use crate::memory::memory_wrapper::MemWrap;
    use log::{info, trace};
    use sdl2::render::Canvas;
    use sdl2::video::Window;
    use std::cell::RefCell;
    use std::fmt::Debug;
    use std::fs::File;
    use std::ops::Sub;
    use std::rc::Rc;
    use std::u8;
    use unit_enum::UnitEnum;
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
        //pub cpu_state: CpuState,
        function_lookup: [FunFind; 63],
        cb_block_lookup: [FunFind; 11],
        //boot_rom_double: File,
        ime_flag: InterruptState,
        stopped: bool,
        clock_cycle_wait: Rc<RefCell<u8>>,
        
        //fetched_instruction:CPUFunct,
        //Preprocess Option<Operand>
        //used for mem reads to HL, failed conditional jumps
        //argument:Argument;
        pub testing_mode: bool,
        pub memory: MemWrap,
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        sp: u16,
        pc: u16,
        command: [u8; 3],
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
                memory: MemWrap::new(joypad, audio, canvas, wait.clone(), cartridge),
                //cpu_state: CpuState::new(joypad, audio, wait.clone(), canvas, cartridge),
                function_lookup: FunFind::function_lookup(),
                cb_block_lookup: FunFind::cb_block_lookup(),
                ime_flag: InterruptState::DisableInterrupt, //Interreupt master enable
                stopped: false,
                clock_cycle_wait: wait,
                testing_mode: false,
                command: [0x00, 0x00, 0x00],
                a: 0x00,
                b: 0x00,
                c: 0x00,
                d: 0x00,
                e: 0x00,
                f: 0x00,
                h: 0x00,
                l: 0x00,
                sp: 0x0000,
                pc: 0x0000,
            } //Find a different way of doing this:
              //Break things apart according to our old pipeline model
        }
        pub fn run_command(&mut self, command: [u8; 3]) {
            //let current_pc = self.get_pc();
            //self.instruction_register = self.memory.grab_memory_8(current_pc);
            //info!("R:{:?}", self);
            self.command = command;
            if self.instruction_register() != 0xF3 {
                //0xf3= disable interrutpts
                self.handle_interrupts();
            }
            if !self.cb_flag() {
                for fun_entry in self.function_lookup {
                    if (self.instruction_register() & fun_entry.mask) == fun_entry.value {
                        match fun_entry.fun {
                            CPUFn::Ld8(src_enum, dest_enum) => self.ld(&src_enum, &dest_enum),
                            CPUFn::Ld16(src16, dest16) => self.ld16(&src16, &dest16),
                            CPUFn::ALU8(alu_func) => {
                                //Where we're storing things, usually A
                                let op = self.alu_operand();
                                self.a = alu_func(self, self.a, op);
                            }
                            CPUFn::ALU8Self(alu_func) => {
                                let op = self.get_r8_end(); //Where we're storing things
                                let val = alu_func(self, op);
                                self.set_r8_end(val);
                            }
                            CPUFn::ALU16Self(alu_fun) => {
                                let reg: DoubleReg = self.r16_tbl();
                                let op = self.get_r16(reg);
                                let val = alu_fun(self, op);
                                self.set_r16(reg, val);
                            }
                            CPUFn::Other(x) => x(self),
                            CPUFn::Cond(x, pc_advance) => {
                                if self.cond() {
                                    x(self);
                                } else {
                                    self.pc += pc_advance as u16;
                                }
                            }
                        };
                        break;
                    }
                }
            } else {
                for fun_entry in &self.cb_block_lookup {
                    if (self.instruction_register() & fun_entry.mask) == fun_entry.value {
                        match fun_entry.fun {
                            CPUFn::ALU8Self(alu_func) => {
                                let reg = self.get_r8_end();
                                let val = alu_func(self, reg);
                                self.set_r8_end(val);
                            }
                            _ => unreachable!(),
                        };
                        break;
                    }
                }

                self.memory.on_clock()
            }
        }
        pub fn interpret_command(&mut self) {
            {
                let mut current_wait = self.clock_cycle_wait.borrow_mut();
                if current_wait.gt(&0) {
                    //println!("We decrement wait by one");
                    *current_wait = current_wait.sub(1);
                }
            }
            if self.clock_cycle_wait.borrow().eq(&0) {
                if !self.stopped {
                    self.command = self.memory.grab_memory_24(self.pc);
                    if self.instruction_register() != 0xF3 {
                        //0xf3= disable interrutpts
                        self.handle_interrupts();
                    }

                    //info!("F:{:X?}", self.f);
                    log::info!("A:{:0>2X?} F:{:0>2X?} B:{:0>2X?} C:{:0>2X?} D:{:0>2X?} E:{:0>2X?} H:{:0>2X?} L:{:0>2X?} SP:{:0>4X?} PC:{:0>4X?} PCMEM:{:0>2X?},{:0>2X?},{:0>2X?},{:0>2X?}",self.a,
                    self.f,
                    self.b, 
                    self.c,
                    self.d,
                    self.e,
                    self.h,
                    self.l,
                    self.sp,
                    self.pc,
                    &self.memory.grab_memory_8(self.pc),
                    &self.memory.grab_memory_8(self.pc+1),
                    &self.memory.grab_memory_8(self.pc+2),
                    &self.memory.grab_memory_8(self.pc+3));
        

                    if !self.cb_flag() {
                        //let current_pc = self.pc;
                        //info!("{:?}", self);
                        //info!("IR:{:X?}", self.command);
                        //info!("Registers {:X?}",self);
                        for fun_entry in self.function_lookup {
                            if (self.instruction_register() & fun_entry.mask) == fun_entry.value {
                                match fun_entry.fun {
                                    CPUFn::Ld8(src_enum, dest_enum) => {
                                        self.ld(&src_enum, &dest_enum)
                                    }
                                    CPUFn::Ld16(src16, dest16) => self.ld16(&src16, &dest16),
                                    CPUFn::ALU8(alu_func) => {
                                        //Where we're storing things, usually A
                                        let op = self.alu_operand();
                                        self.a = alu_func(self, self.a, op);
                                    }
                                    CPUFn::ALU8Self(alu_func) => {
                                        let op = self.get_r8_end(); //Where we're storing things
                                        let val = alu_func(self, op);
                                        self.set_r8_end(val);
                                    }
                                    CPUFn::ALU16Self(alu_fun) => {
                                        let reg: DoubleReg = self.r16_tbl();
                                        let op = self.get_r16(reg);
                                        let val = alu_fun(self, op);
                                        self.set_r16(reg, val);
                                    }
                                    CPUFn::Other(x) => x(self),
                                    CPUFn::Cond(x, pc_advance) => {
                                        if self.cond() {
                                            x(self);
                                        } else {
                                            self.pc += pc_advance;
                                        }
                                    }
                                };
                                self.wait(fun_entry.wait);
                                break;
                            }
                        }

                    /* if current_pc==self.pc{
                        self.pc+=1
                    } */
                    } else {
                        let lookup = &self.cb_block_lookup;
                        for fun_entry in lookup {
                            if (self.instruction_register() & fun_entry.mask) == fun_entry.value {
                                match fun_entry.fun {
                                    CPUFn::ALU8Self(alu_func) => {
                                        let op = self.get_r8_end(); //Where we're storing things
                                        let val = alu_func(self, op);
                                        self.set_r8_end(val);
                                    }
                                    _ => unreachable!(),
                                };
                                break;
                            }
                        }
                        self.pc += 1;
                    }
                } else {
                    self.handle_interrupts(); //We must handle interrutps for stop case.
                }
                self.pc += 1;
            }
        }

        pub fn on_clock(&mut self) {
            self.interpret_command();
            self.memory.on_clock()
        }
        fn cb_flag(&self) -> bool {
            self.command[0] == 0xCB
        }
        pub fn wait(&mut self, cycles: u8) {
            //We need a way to model this such that we prefix our waits instead of postfixing them.
            *self.clock_cycle_wait.borrow_mut() += cycles;
        }
        pub fn test_init(&mut self) {
            self.a = 0x01;
            self.f = 0xB0;
            self.c = 0x13;
            self.e = 0xd8;
            self.h = 0x01;
            self.l = 0x4d;
            self.pc = 0x0100;
            self.sp = 0xFFFE;
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
            let enabled_interrupts_flag: u8 = self.memory.grab_memory_8(0xFFFF);
            let mut bit_idx: u8 = 1;
            let mut target_call: u16 = 0x0040;
            let interrupt_flag: u8 = self.memory.grab_memory_8(0xFF0F);
            loop {
                if (interrupt_flag & enabled_interrupts_flag & bit_idx) > 0 {
                    //We process an interrupt
                    if are_interreupts_enabled {
                        //We check in here so that if we have the case where we're halted, and don't have IME Enabled, we unhalt
                        self.memory.set_memory_8(0xFF0F, interrupt_flag ^ bit_idx); //unset bit
                        self.ime_flag = InterruptState::Disabled;
                        self.push_stack(self.pc);
                        self.pc = target_call;
                    }
                    self.stopped = false; //Implement the Halt bug!
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
        fn instruction_register(&self) -> u8 {
            if self.cb_flag() {
                return self.command[1];
            }
            self.command[0]
        }
        pub fn cond(&mut self) -> bool {
            //info!("WE'RE ASSESSING CONDITION {:?} ", (opcode >> 4) % 4);
            let zero = self.get_flag(Flag::Zero);
            let carry = self.get_flag(Flag::Carry);
            match (self.instruction_register() >> 3) % 4 {
                0 => !zero,
                1 => zero,
                2 => !carry,
                3 => carry,
                _ => unreachable!(),
            }
        }
        pub fn alu_operand(&mut self) -> u8 {
            match self.instruction_register() > 0xC0 {
                true => self.get_imm8(),
                false => self.get_r8_end(),
            }
        }
        pub fn nop(&mut self) {
            () //info!("CPU OP: NOP");
        }
        // Rotations
        pub fn rl(&mut self, op: u8) -> u8 {
            //info!("CPU Operation is Rotate Left through Carry");
            let carry: bool = self.get_flag(Flag::Carry);
            let ret = (op << 1) + (carry as u8);
            self.set_flags(self.cb_flag() && ret == 0, false, false, op > 127);
            ret
        }

        pub fn rlc(&mut self, op: u8) -> u8 {
            //info!("CPU Operation is Rotate Left circular");
            self.set_flags(self.cb_flag() && op == 0, false, false, op > 127);
            op.rotate_left(1)
        }
        pub fn rr(&mut self, op: u8) -> u8 {
            //info!("CPU Operation is Rotate Right");
            let carry: bool = self.get_flag(Flag::Carry);
            let bottom: bool = (op % 2) == 1;
            let ret = (op >> 1) + ((carry as u8) << 7);
            self.set_flags(self.cb_flag() && ret == 0, false, false, bottom);
            ret
        }
        pub fn rrc(&mut self, op: u8) -> u8 {
            //info!("CPU Operation is Rotate Right circular");
            self.set_flags(self.cb_flag() && op == 0, false, false, (op % 2) == 1);
            op.rotate_right(1)
        }

        pub fn daa(&mut self, acc: u8) -> u8 {
            //info!("CPU Operation is Decimal Adjust Accumulator");
            let subtract = self.get_flag(Flag::Neg);
            let hcarry = self.get_flag(Flag::HalfCarry);
            let carry = self.get_flag(Flag::Carry);
            let mut offset: u8 = 0;
            if (!subtract && (self.a & 0xf > 0x9)) || hcarry {
                offset |= 0x06;
            }
            if (!subtract && (self.a > 0x99)) || carry {
                offset |= 0x60;
            }
            let ret = match subtract {
                true => acc.wrapping_sub(offset),
                false => acc.wrapping_add(offset),
            };
            self.mark_flag(Flag::Zero, ret == 0);
            self.mark_flag(Flag::HalfCarry, false);
            self.mark_flag(Flag::Carry, ret > 0x99);
            ret
        }
        pub fn ld_hl_imm8(&mut self) {
            //info!("CPU OP:LOAD SP+IMM -> HL");
            let imm = self.get_imm8();
            self.set_r16(DoubleReg::HL, self.sp - (0x100 - (imm as u16)));
        }
        pub fn ld16(&mut self, source: &Src16, dest: &Dest16) {
            let value: u16 = match source {
                Src16::Imm16 => self.get_imm16(),
                Src16::HL => self.get_r16(DoubleReg::HL),
                Src16::SP => self.get_r16(DoubleReg::SP),
            };
            //info!("{:X?}",value);
            match dest {
                Dest16::R16 => {
                    let reg = self.r16_tbl();
                    self.set_r16(reg, value);
                }
                Dest16::PC => self.set_r16(DoubleReg::PC, value),
                Dest16::SP => self.set_r16(DoubleReg::SP, value),
                Dest16::HL => self.set_r16(DoubleReg::HL, value),
            }
        }
        pub fn ld(&mut self, source: &Src8, dest: &Dest8) {
            let source_value: u8 = match source {
                Src8::Imm16Mem => {
                    let imm = self.get_imm16();
                    self.memory.grab_memory_8(imm)
                }
                Src8::Imm8 => self.get_imm8(),
                Src8::HighBank => {
                    let imm = self.get_imm8();
                    //print!("{}",imm);
                    let ret = self.memory.grab_memory_8(u16::from_be_bytes([0xff, imm]));
                    //print!("{}",ret);
                    ret
                }
                Src8::Acc => self.a,
                Src8::R8Mid => self.get_r8_mid(),

                Src8::R8 => self.get_r8_end(),
                Src8::HighC => self
                    .memory
                    .grab_memory_8(u16::from_be_bytes([0xff, self.c])),
                Src8::R16Mem => {
                    let reg = self.r16_mem_tbl();
                    self.get_r16_mem_8(reg)
                }
            };
            match dest {
                Dest8::R8 => self.set_r8_mid(source_value),
                Dest8::Imm16Mem => {
                    let dest = self.get_imm16();
                    self.memory.set_memory_8(dest, source_value);
                }
                Dest8::Imm8High => {
                    let imm = self.get_imm8();
                    self.memory
                        .set_memory_8(u16::from_be_bytes([0xff, imm]), source_value);
                }
                Dest8::Acc => self.a = source_value,
                Dest8::R16Mem => {
                    self.set_r16_mem_8(self.r16_mem_tbl(), source_value);
                }
                Dest8::HighC => self
                    .memory
                    .set_memory_8(u16::from_be_bytes([0xff, self.c]), source_value),
            }
        }

        pub fn inc_r8(&mut self) {
            //info!("CPU OP:INC R8");
            let reg = self.get_r8_mid();
            self.set_flags2([Some(reg == 0xff), Some(false), Some(reg %0x10 == 0xf), None]);
            self.set_r8_mid(reg.wrapping_add(1));
        }
        pub fn dec_r8(&mut self) {
            let reg = self.get_r8_mid();
            self.mark_flag(Flag::Zero, reg == 1);
            self.mark_flag(Flag::Neg, true);
            self.mark_flag(Flag::HalfCarry, reg == 0x10);
            self.set_r8_mid(reg.wrapping_sub(1));
        }
        pub fn inc_r16(&mut self, op: u16) -> u16 {
            //info!("CPU OP:Increment an R16");
            op.wrapping_add(1)
        }
        pub fn dec_r16(&mut self, op: u16) -> u16 {
            //info!("CPU OP:Decrement an R16");
            op.wrapping_sub(1)
        }
        pub fn add_hl(&mut self) {
            //info!("CPU OP:Increment HL");
            let operand: u16 = self.get_r16(self.r16_tbl());
            let hl_val: u16 = self.get_r16(DoubleReg::HL);
            let result = hl_val.wrapping_add(operand);
            self.set_r16(DoubleReg::HL, result);
            self.set_flags2([
                None,
                Some(false),
                Some((hl_val & 0x0fff) + (operand & 0x0fff) > 0x1000),
                Some(hl_val.checked_add(operand).is_none()),
            ]);
        }
        pub fn cpl(&mut self) {
            //info!("CPU OP: Invert A");
            self.mark_flag(Flag::HalfCarry, true);
            self.mark_flag(Flag::Neg, true);
            self.a = !self.a;
        }
        pub fn ccf(&mut self) {
            self.mark_flag(Flag::Neg, false);
            self.mark_flag(Flag::HalfCarry, false);
            self.f ^= Flag::Carry.discriminant();
        }
        pub fn scf(&mut self) {
            self.f |= Flag::Carry.discriminant();
        }
        pub fn add(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Add without carry");
            let ret = acc.wrapping_add(op);
            self.set_flags(
                ret == 0,
                false,
                ((acc & 0x0F) + (op & 0x0F)) > 0x10,
                acc.checked_add(op).is_none(),
            );
            ret
        }
        pub fn adc(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Add with carry");
            let carry = self.get_flag(Flag::Carry);
            let operand = op + (carry as u8);
            let ret = acc.wrapping_add(operand);
            self.set_flags(
                ret == 0,
                false,
                ((acc & 0x0F) + (operand & 0x0F)) > 0x10,
                acc.checked_add(operand).is_none(),
            );
            ret
        }
        pub fn sub(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Sub without carry");
            self.set_flags(acc == op, true, (acc & 0x0F) < (op & 0x0F), op > acc);
            acc.wrapping_sub(op)
        }
        pub fn subc(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Sub with Carry");
            let carry: u8 = self.get_flag(Flag::Carry) as u8;
            let operand: u8 = op + carry;
            self.set_flags(
                acc == operand,
                true,
                (acc & 0x0F) < (operand & 0x0F),
                operand > acc,
            );
            acc.wrapping_sub(operand)
        }
        pub fn and(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: And");
            self.set_flags(op & acc == 0, false, true, false);
            acc & op
        }
        pub fn xor(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Xor");;
            //println!("{:?}",acc^op);
            self.set_flags((acc ^ op) == 0, false, false, false);
            acc ^ op
        }
        pub fn or(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Or");
            self.set_flags(acc | op == 0, false, false, false);
            acc | op
        }
        pub fn cp(&mut self, acc: u8, op: u8) -> u8 {
            //info!("CPU OP: Compare");
            self.set_flags(acc == op, true, (acc & 0x0F) < (op & 0x0F), op > acc);
            acc
        }
        pub fn sla(&mut self, op: u8) -> u8 {
            //info!("CPU OP: Shift left into carry");
            self.set_flags(op == 128 || op == 0, false, false, op > 127);
            op << 1
        }
        pub fn sra(&mut self, op: u8) -> u8 {
            //info!("CPU OP: Shift right into carry");
            self.set_flags(op < 2, false, false, op % 2 == 1);
            //Sneaky little arithmetic right shift.
            (op >> 1) + (128 * ((op > 127) as u8))
        }
        pub fn srl(&mut self, op: u8) -> u8 {
            self.set_flags(op < 2, false, false, op % 2 == 1);
            op >> 1
        }
        pub fn swap(&mut self, op: u8) -> u8 {
            //info!("CPU OP: Swap nibbles");
            self.set_flags(op == 0, false, false, false);
            op.rotate_left(4)
        }
        pub fn bit_idx(&self) -> u8 {
            (self.instruction_register() & 0x3F) >> 3
        }
        pub fn bit(&mut self, op: u8) -> u8 {
            //info!("CPU OP: CHECK BIT");
            self.mark_flag(Flag::Zero, ((op >> self.bit_idx()) % 2) == 0);
            self.mark_flag(Flag::Neg, false);
            self.mark_flag(Flag::HalfCarry, true);
            op
        }
        pub fn res(&mut self, op: u8) -> u8 {
            //info!("CPU OP: RESET BIT");
            op & !(1 << self.bit_idx())
        }
        pub fn set(&mut self, op: u8) -> u8 {
            //info!("CPU OP: SET BIT");
            op | (1 << self.bit_idx())
        }
        pub fn ret(&mut self) {
            self.pc = self.pop_stack()-1;
        }
        pub fn reti(&mut self) {
            //info!("CPU OP: Return, Enable Interrupts");
            self.pc = self.pop_stack();
            self.ime_flag = match self.ime_flag {
                InterruptState::Disabled => InterruptState::EnableInterrupt,
                InterruptState::AlmostEnabled | InterruptState::Enabled => InterruptState::Enabled,
                InterruptState::EnableInterrupt => unreachable!(),
                InterruptState::DisableInterrupt => unreachable!(),
            }
        }
        pub fn jr_imm(&mut self) {
            let imm = self.get_simm8() as i8;
            self.pc = self.pc.wrapping_add_signed(imm.into());
        }
        pub fn jump(&mut self) {
            let imm: u16 = self.get_imm16();
            self.pc = imm - 1;
        }
        pub fn jp_cond_imm(&mut self) {
            //info!("CPU OP: Jump Conditional To IMM16");
            let imm: u16 = self.get_imm16(); //This way we increment the PC
            if self.cond() {
                self.pc = imm - 1;
            }
        }
        pub fn call(&mut self) {
            //info!("CPU OP: Call IMM");
            self.push_stack(self.pc + 3);
            self.pc = self.get_imm16()-1; //This fixed a call bug
        }
        pub fn rst(&mut self) {
            //info!("OP: RST");
            self.push_stack(self.pc);
            self.pc = ((self.instruction_register() & 0x38) - 1) as u16;
        }
        pub fn pop(&mut self) {
            let pop_val = self.pop_stack();
            self.set_r16(self.r16_stk_tbl(), pop_val); //WE ADD ONE TO OUR POP VALUE?
        }
        pub fn push(&mut self) {
            let value: u16 = self.get_r16(self.r16_stk_tbl());
            self.push_stack(value);
        }
        pub fn add_sp_imm8(&mut self) {
            let operand: i8 = self.get_simm8();
            self.mark_flag(
                Flag::Carry,
                self.sp.checked_add_signed(operand as i16).is_none(),
            );
            //self.mark_flag(Flag::HalfCarry, ((acc & 0x000F).saturating_add(operand & 0x0F)) > 0x10 );
            self.mark_flag(Flag::Neg, false);
            self.mark_flag(Flag::Zero, false);
            self.sp = self.sp.wrapping_add_signed(operand as i16);
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
        pub fn get_r8_mid(&mut self) -> u8 {
            match (self.instruction_register() >> 3) % 8 {
                0 => self.b,
                1 => self.c,
                2 => self.d,
                3 => self.e,
                4 => self.h,
                5 => self.l,
                6 => self.get_r16_mem_8(DoubleReg::HL),
                7 => self.a,
                _ => unreachable!(),
            }
        }
        pub fn set_r8_mid(&mut self, val: u8) {
            match (self.instruction_register() >> 3) % 8 {
                0 => self.b = val,
                1 => self.c = val,
                2 => self.d = val,
                3 => self.e = val,
                4 => self.h = val,
                5 => self.l = val,
                6 => self.set_r16_mem_8(DoubleReg::HL, val),
                7 => self.a = val,
                _ => unreachable!(),
            }
        }
        pub fn set_r8_end(&mut self, val: u8) {
            match self.instruction_register() % 8 {
                0 => self.b = val,
                1 => self.c = val,
                2 => self.d = val,
                3 => self.e = val,
                4 => self.h = val,
                5 => self.l = val,
                6 => self.set_r16_mem_8(DoubleReg::HL, val),
                7 => self.a = val,
                _ => unreachable!(),
            }
        }

        pub fn r16_mem_tbl(&self) -> DoubleReg {
            //(trace!("TESTING"),DoubleReg::HLM).1;
            match (self.instruction_register() >> 4) % 4 {
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HLP,
                3 => DoubleReg::HLM,
                _ => unreachable!(),
            }
            
        }
        fn get_r8_end(&mut self) -> u8 {
            match self.instruction_register() % 8 {
                0 => self.b,
                1 => self.c,
                2 => self.d,
                3 => self.e,
                4 => self.h,
                5 => self.l,
                6 => self.get_r16_mem_8(DoubleReg::HL),
                7 => self.a,
                _ => unreachable!(),
            }
        }
        fn r16_tbl(&self) -> DoubleReg {
            match (self.instruction_register() >> 4) % 4 {
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::SP,
                _ => unreachable!(),
            }
        }
        fn r16_stk_tbl(&self) -> DoubleReg {
            //println!("{:x}", self.instruction_register());
            match (self.instruction_register() >> 4) % 4 {
                0 => DoubleReg::BC,
                1 => DoubleReg::DE,
                2 => DoubleReg::HL,
                3 => DoubleReg::AF,
                _ => unreachable!(),
            }
        }
        fn set_flags(&mut self, zero: bool, neg: bool, hc: bool, carry: bool) {
            self.f = (128 * zero as u8) + (64 * neg as u8) + (32 * hc as u8) + 16 * (carry as u8);
            //println!("{:X?}",self.f);
        }
        fn set_r16_mem_8(&mut self, reg: DoubleReg, val: u8) {
            let reg_val = self.get_r16(reg);
            self.memory.set_memory_8(reg_val, val)
        }
        fn get_r16_mem_8(&mut self, reg: DoubleReg) -> u8 {
            let addr = self.get_r16(reg);

            let ret = self.memory.grab_memory_8(addr);

            //info!("GETTING MEMORY! {addr:#x} {ret:#x}");
            ret 
        }
        fn pop_stack(&mut self) -> u16 {
            let value = self.memory.grab_memory_16(self.sp);
            self.sp = self.sp + 2;
            value
        }
        fn push_stack(&mut self, val: u16) {
            self.sp = self.sp - 2;
            self.memory.set_memory_16(self.sp, val);
        }
        fn get_flag(&self, flag: Flag) -> bool {
            self.f & flag.discriminant() == flag.discriminant()
        }
        fn mark_flag(&mut self, flag: Flag, state: bool) {
            if state {
                self.f |= flag.discriminant()
            } else {
                self.f &= !flag.discriminant()
            }
        }
        fn get_imm8(&mut self) -> u8 {
            self.pc += 1;
            self.command[1]
        }
        fn get_simm8(&mut self) -> i8 {
            self.pc += 1;
            self.command[1] as i8
        }
        fn get_imm16(&mut self) -> u16 {
            self.pc += 2;
            u16::from_be_bytes([self.command[2], self.command[1]])
        }
        fn get_r16(&mut self, reg: DoubleReg) -> u16 {
            let glue = |x: u8, y: u8| x as u16 * 0x100 + y as u16;
            let result = match reg {
                DoubleReg::AF => glue(self.a, self.f),
                DoubleReg::BC => glue(self.b, self.c),
                DoubleReg::DE => glue(self.d, self.e),
                DoubleReg::HL => glue(self.h, self.l),
                DoubleReg::HLP => {
                    let mut addr = glue(self.h, self.l);
                    addr += 1;
                    self.h = (addr >> 8) as u8;
                    self.l = addr as u8;
                    addr-1
                }
                DoubleReg::HLM => {
                    let mut addr = glue(self.h, self.l);
                    addr -= 1;
                    self.h = (addr >> 8) as u8;
                    self.l = addr as u8;
                    addr + 1
                }
                DoubleReg::SP => self.sp,
                DoubleReg::PC => self.pc,
            };
            if (result== 0xc182){
                //info!("HEY WE FOUND IT")
            }
            return result
        }

        fn set_r16(&mut self, reg: DoubleReg, val: u16) {
            match reg {
                DoubleReg::HLP | DoubleReg::HLM => {
                    panic!("We can't set HLPM from Double Set")
                }
                DoubleReg::AF => {
                    //panic!("convenient");
                    self. a = (val >> 8) as u8;
                    self.f = (val as u8) &0xf0;
                }
                DoubleReg::BC => {
                    self.b = (val >> 8) as u8;
                    self.c = val as u8;
                }
                DoubleReg::DE => {
                    self.d = (val >> 8) as u8;
                    self.e = val as u8;
                }
                DoubleReg::HL => {
                    println!("{:x}",val);
                    self.h = (val >> 8) as u8;
                    self.l = val as u8;
                }
                DoubleReg::SP => self.sp = val,
                DoubleReg::PC => self.pc = val,
            }
        }

        fn set_flags2(&mut self, flags: [Option<bool>; 4]) {
            let mut idx = 128;
            for i in flags {
                match i {
                    Some(val) => match val {
                        true => self.f |= idx,
                        false => self.f &= !idx,
                    },
                    None => (),
                }
                idx = idx >> 1;
            }
        }
    }
    #[derive(UnitEnum)]
    #[repr(u8)]
    pub enum Flag {
        Zero = 0x80,
        Neg = 0x40, //Often marked as N.
        HalfCarry = 0x20,
        Carry = 0x10,
    }
    #[derive(Copy, Clone, Debug)]
    pub enum SingleReg {
        A,
        B,
        C,
        D,
        E,
        F,
        H,
        L,
        Memptr,
    }
    #[derive(Copy, Clone, Debug)]
    pub enum DoubleReg {
        AF,
        BC,
        DE,
        HL,
        HLP,
        HLM,
        SP,
        PC,
    }
    impl Debug for CpuStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("")
            .field("A", &format!("{:0>2X?}", &self.a))
            .field("F", &format!("{:0>2X?}", &self.f))
            .field("B", &format!("{:0>2X?}", &self.b))
            .field("C", &format!("{:0>2X?}", &self.c))
            .field("D", &format!("{:0>2X?}", &self.d))
            .field("E", &format!("{:0>2X?}", &self.e))
            /*                 .field(
                "F",
                &format!(
                    "[Z:{},N:{},HC:{},C:{}]",
                    ((&self.f >> 7) == 1) as u8,
                    (((&self.f & 0x40) >> 6) == 1) as u8,
                    (((&self.f & 0x20) >> 5) == 1) as u8,
                    (((&self.f & 0x10) >> 4) == 1) as u8
                ),
            ) */
            .field("H", &format!("{:0>2X?}", &self.h))
            .field("L", &format!("{:0>2X?}", &self.l))
            .field("SP", &format!("{:0>4X?}", &self.sp))
            .field("PC", &format!("{:0>4X?}", &self.pc))
            .finish()
            
        }
    }
    
    pub enum Interrupt {
        VBlank,
        LCDC,
        Timer,
        Serial, //Unimplemented
        Input,
    }
}
