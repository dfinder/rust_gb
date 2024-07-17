use cpu; 
mod instruction_set{
    enum instr_val_bit{
        0,
        1,
        dest_16,
        dest_16_mem,
        r8,
        r16,
        r16stk,
        flag,
        b3, 
        tgt3,
    }
    enum node_serach_bit{
        0,
        1,
        Either
    }
    public struct instruction{ 
        len:u8 //1->3
        val:[inst_val_bit; 8]
        stall: u8
        function: fn(cpu);
    }
    

    impl instruction_set{
        fn possible_instructions(inst: instruction) -> Vec<u8>{
            for i in val{

            }
        }
        fn setup_instructions(&self){

        }
        fn find_instruction(){
           
        }
    }
}