use std::{thread,time};
mod cpu { 
    mod registers { 
        pub struct register_set{
            i8 A; //A IS THE ACCUMULATOR
            i8 B;
            i8 C;
            i8 D;
            i8 E; 
            i8 F; 
            i8 H;
            i8 L;
            i16 SP; 
            i16 PC;  
        }
        impl registers{
            fn AF(&mut self)->i16{
                self.A * 2<<8 + self.F;
            }
            fn BC(&mut self)->i16{
                self.B * 2<<8 + self.C;
            }
            fn DE(&mut self)->i16{
                self.D * 2<<8 + self.E;
            }
            fn HL(&mut self)->i16{
                self.H * 2<<8 + self.L;
            }
            fn set_zero(&mut self){
                self.F |= 1;
            }
            fn get_zero(&mut self )-> bool{
                self.F & 1;
            }
            //Can parallelize this to other flags, leave for later.

        }
        fn build_registers(){

        }
    }
    mod built_in_ram{
        impl ram{

        }
    }
    mod cpu{
        const CLOCK_PERIOD: time::duration = Duration::from_nanos(239)
        public struct cpu{
            register_set registers;
        }
        fn instantiate_cpu(){

        }
        fn interpret_command(&mut self,&mut memory){
            my_pc = self.register_set.PC;
            current_command = memory.grab_memory_8(my_pc);
            //We can't use a normal match case here because rust doesn't support the sort of work
            if current_command == 0{
                noop()
            }
            if (current_command & 0b11001111)==1{
                load_register_immediate()//following byte
            }
            if (current_command & 0b11001111) == 1{
                
            }
        }
        fn cd_block()
        fn wait(i8 cycles){
            //4.19 mhz * 4 t cycles 
            thread.sleep(4*CLOCK_PERIOD*cycles);
        }

    }



}