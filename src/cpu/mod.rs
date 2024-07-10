use std::{thread,time};
mod cpu { 
    pub fn set_bit(x: u8, idx: u8, b: bool) -> u8 { // should probably make some file that's just helper functions.
        let mask = !(1 << idx);
        let flag = (b as u8) << idx;
        x & mask | flag
    }
    pub fn sign_8(x: u8)-> i8{
        (!x)+1
    }
    mod registers { 
        pub struct register_set{
            u8 A; //A IS THE ACCUMULATOR
            u8 B;
            u8 C;
            u8 D;
            u8 E; 
            u8 F; 
            u8 H;
            u8 L;
            u16 SP; 
            u16 PC;  
        }
        impl registers{
            fn AF(&mut self)->u16{
                self.A * 1<<8 + self.F;
            }
            fn BC(&mut self)->u16{
                self.B * 1<<8 + self.C;
            }
            fn DE(&mut self)->u16{
                self.D * 1<<8 + self.E;
            }
            fn HL(&mut self)->u16{
                self.H * 1<<8 + self.L;
            }
            fn set_zero(&mut self){
                self.F |= 1; //OR EQUALS
            }
            fn get_zero(&mut self )-> bool{
                self.F & 1;
            }
            fn get_carry(&mut self)->bool{
                (self.F & 8 == 8) 
            }
            fn increment_pc(&mut self,u8 amount) -> u16{
                self.PC += amount
            }
            fn increment_pc(&mut self) -> u16{
                self.PC += 1
            }
            fn set_carry(&mut self, bool value){
                self.F = cpu.set_bit(self.F,3,value)
            }

            //Can parallelize this to other flags, leave for later.

        }
        fn build_registers(current_command){

        }
    }
    mod built_in_ram{
        impl ram{

        }
    }
    mod cpu{
        const CLOCK_PERIOD: time::duration = Duration::from_nanos(239)
        public struct cpu{
            register_set registers = registers.build_registers(current_command);
            memory memory_reference;
        }
        fn instantiate_cpu() -> Self{

        }
        fn interpret_command(&mut self){
            //my_pc = self.register_set.PC;
            current_command:u8 = self.memory.grab_memory_8(self.register_set.PC);
            first_two:u8 = current_command > 6
            match first_two{
                0 => block_0(current_command)
                1 => load(current_command)
                2 => arithmetic(current_command)
                3 => block_3(current_command)
        }
        fn block_0(&mut self, &mut current_command){
            match current_command{
                0 => self.no_op()
                7 => self.rlca()
                8 => self.load_sp()
                15 => self.rrca()
                16 => self.stop()
                23 => self.rla()
                24 => self.jr()
                31 => self.rra()
                39 => self.daa()
                47 => self.cpl()
                55 => self.scf()
                63 => self.cff()
                _ => self.b0_read_registers(current_command)
            }
        }
        fn no_op(&mut self){
            self.wait(1);
        }
        fn rlca(&mut self){ //Rotate register A to the left. Set carry to whatever bit 7 was.
            self.registers.carry((self.registers.A>>7>0)); 
            self.registers.A = self.registers.A.rotate_left(1);
            self.wait(1);
        }
        fn rla(&mut self){ //Rotate register a to the left _through_ the carry bti .
            carry:bool = self.registers.get_carry();
            top:bool = self.registers.A & 0x80 == 0x80
            self.registers.set_carry(top)
            self.registers.A = self.registers.A << 1 + carry
            self.wait(1) 
        }
        fn load_sp(&mut self){
            self.registers.SP = self.memory.grab_memory_16(self.registers.PC+1);
            self.registers.increment_pc(2);
            self.wait(3);
        }
        fn rrca(&mut self){
            self.registers.set_carry((self.registers.A<<7)>0)); 
            self.registers.A = self.registers.A.rotate_right(1);
            self.wait(1);
        }
        fn stop(&mut self){
            panic!("crash and burn");
        }
        fn jr(&mut self){
            next_value: 18 = (self.memory.grab_memory_8(self.registers.PC+1) as i8);
            if (next_value>0){
                self.registers.PC += next_value as u8 
                   //but wait, this is signed
            }
            else{
                self.registers.PC -= (!(next_value) + 1) as u8 
            }
            self.wait(3)
        }
        fn cd_block()
        fn wait(i8 cycles){
            //4.19 mhz * 4 t cycles 
            thread.sleep(4*CLOCK_PERIOD*cycles);
        }

    }



}