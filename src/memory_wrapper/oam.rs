pub mod oam{
    use crate::memory_wrapper::memory_wrapper::AsMemory;

    impl AsMemory for OamStruct{
        fn memory_map(&mut self,addr:u16)->u8 {
            todo!()
        }
    
        fn memory_write(&mut self,addr:u16,val:u8) {
            todo!()
        }
    }
    pub struct OamStruct{

    }
}