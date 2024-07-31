pub mod memory_wrapper{
    use crate::memory::memory::MemoryStruct;

    pub struct MemWrap{
        memory_reference: MemoryStruct,
    }
    impl MemWrap{
        pub fn new()->Self{
            Self { memory_reference: MemoryStruct::new() }
        }
        pub fn grab_memory_8(&mut self, addr:u16)->u8{
            self.memory_reference.grab_memory_8(addr)
        }
        pub fn grab_memory_16(&mut self,addr:u16)->u16{
            self.memory_reference.grab_memory_16(addr)
        }
        pub fn set_memory_8(&mut self, addr:u16, value:u8){
            self.memory_reference.set_memory_8(addr,value)
        }
        pub fn set_memory_16(&mut self, addr:u16, value:u16){
            self.memory_reference.set_memory_16(addr,value)
        }
    }
}