pub mod ex_rom {
    use crate::memory_wrapper::memory_wrapper::AsMemory;

    pub struct ExROM {

    }
    impl ExROM{
        pub fn new()->Self{
            return Self{}
        }
    }

    impl AsMemory for ExROM{
        fn memory_map(&mut self, addr: u16) -> u8 {
            todo!()
        }
    
        fn memory_write(&mut self, addr: u16, val: u8) {
            todo!()
        }
    }
}
