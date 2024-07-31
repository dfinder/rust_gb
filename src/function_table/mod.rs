
pub mod function_table{
    use crate::cpu::cpu::CpuStruct;
    use crate::cpu::cpu::CPUFunct;
    //#[derive(Copy)]
    pub struct FunFind{
        pub mask:u8,
        pub value:u8,
        pub function: CPUFunct,//,argument Arg. returns false if we have a longer wait.
        pub wait:u8,
        pub wait_cond:Option<u8>,
        //flags: FlagS,
        //bytes: u8//1,2,3, measures the enums.
    }
    impl FunFind{
        pub fn fun_find(mask: u8, value: u8, function:CPUFunct, wait:u8)->Self{
            Self{
                mask,
                value,
                function,
                wait,
                wait_cond:None,
            }
        }
        pub fn fun_find_w(mask: u8, value: u8, function:CPUFunct, wait:u8, wait_cond:u8 )->Self{
            Self{
                mask: mask,
                value:value,
                function:function,
                wait:wait,
                wait_cond: Some(wait_cond),
            }
        }
    }

        
}