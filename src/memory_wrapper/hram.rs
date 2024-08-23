pub mod hram{
    use crate::memory_wrapper::memory_wrapper::AsMemory;

    pub struct HRam{
        
    }
    impl AsMemory for HRam{
        fn memory_map(&mut self,addr:u16)->u8 {
            todo!()
        }
    
        fn memory_write(&mut self,addr:u16,val:u8) {
            todo!()
        }
    }
}