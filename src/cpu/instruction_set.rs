use registers; //This was a waste of time tbh.
mod instruction_set{

    //public struct instruction{ 
    //    len:u8; //1->3 //Essentially marks imm8, imm16
    //    val: opcode
    //    stall: u8
    //    option_stall: Option<u8>
    //    function: Fn(cpu)
    //}
    //impl execute for instruction{
    //    fn run(&self, proc: &cpu){
    //       function(proc)
    //
    //    }
    //}
    struct instruction_set{
        list_of_instructions: [instruction;255]
    }

    impl instruction_set{
        fn possible_instructions(inst: opcode::complex) -> Vec<u8>{
            let result: Vec<u8> = Vec::new(); 
            let find_one = | i:u8 | -> u8 {i<<1 +1}
            let find_zero = | i:u8 | -> u8 {i << 1}
            let find_two_bit = | i:u8 | -> [u8;4] {[i<<2, (i<<2)+1, (i<<2)+2, (i<<2)+3]}
            let find_three_bit = | i:u8 | -> [u8;8] {[(i<<3),(i<<3)+1,(i<<3)+2,(i<<3)+3,(i<<3)+4,(i<<3)+5,(i<<3)+6,(i<<3)+7]}
            for i in inst.val{
                match i{
                    0 => {if result.len()==0 {result.push(0)} else {result=result.iter().map(find_zero).collect::<u8>()}}
                    1 => {if result.len()==0 {result.push(1)} else result=result.iter().map(find_one).collect::<u8>()}
                    r16|r16stk|r16mem|flag => result = result.iter().map(find_two_bit).flatten().collect::<u8>
                    r8|b3|tgt3 => result = result.iter().map(find_three_bit).flatten().collect::<u8>
                }
            }
            return result;
        }
        fn setup_instructions(&self){


        }
        fn find_instruction(opcode:u8){
        
        }
        fn nop(){
           self.wait(1)
        }

        fn rlca(&mut processor:cpu){ //Rotate register A to the left. Set carry to whatever bit 7 was.
            //processor.registers.carry((processor.registers.A>>7>0)); 
            //processor.registers.A = processor.registers.A.rotate_left(1);
            //self.wait(1);
            let carry_value : bool = (processor.registers.get_acc() >> 7) > 0
            processor.registers.set_flag(registers::Flag::Carry, carry_value)
            self.wait(1)
        }
        fn rla(&mut processor:cpu){ //Rotate register a to the left _through_ the carry bti .
            let carry:bool = processor.registers.get_carry();
            let top:bool = processor.registers.A & 0x80 == 0x80
            processor.registers.set_carry(top)
            processor.registers.A = processor.registers.A << 1 + carry
            self.wait(1) 
        }
        fn load_sp(&mut self){
            processor.registers.SP = processor.memory.grab_memory_16(processor.registers.PC+1);
            processor.registers.increment_pc(2);
            self.wait(3);
        }
        fn rrca(){
            processor.registers.set_carry((processor.registers.A<<7)>0)); 
            processor.registers.set_single_register(Registers:SingleRegister::A, )= processor.registers.A.rotate_right(1);
            self.wait(1);
        }
        fn stop(&mut self){
            panic!("crash and burn");
        }
        fn rra(&mut self){
             //Rotate register a to the right _through_ the carry bti .
                let carry:bool = processor.registers.get_carry();
                let bottom:bool = (processor.registers.A & 0x01) == 0x01 //Get bit number 8W
                processor.registers.set_carry(bottom)
                processor.registers.A = processor.registers.A > 1 + carry << 8
                self.wait(1)
        }
        fn jr(&mut processor:cpu){
            let next_value: 18 = (self.memory.grab_memory_8(processor.registers.PC+1) as i8);
            if (next_value>0){
                processor.registers.PC += (next_value as u8)
                   //but wait, this is signed
            }
            else{
                processor.registers.PC -= (!(next_value) + 1) as u8 
            }
            self.wait(3)
        }
        fn wait(i8 cycles){
            //4.19 mhz * 4 t cycles 
            thread.sleep(4*CLOCK_PERIOD*cycles);
        }
    }
}