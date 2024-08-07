pub mod memory_wrapper{
    use crate::memory_wrapper::mapped_io::mapped_io;
    use crate::memory::memory::MemoryStruct;
    use crate::joypad::joypad::*;
    pub struct MemWrap{
        memory_reference: MemoryStruct,
    }
    impl MemWrap{
        pub fn new()->Self{
            Self { memory_reference: MemoryStruct::new() }
        }
        fn memory_map(&mut self,addr:u16){
            match addr{
                0x0000..=0x7FFF => todo!(),//ROM
                0x8000..=0x9FFF => todo!(),//VRAM
                0xA000..=0xBFFF => todo!(),//External bus
                0xC000..=0xDFFF => todo!(), //WRAM
                0xE000..=0xFDFF => todo!(), //ECHO
                0xFE00..=0xFE9F => todo!(), // OAM
                0xFEA0..=0xFEFF => todo!(),//Invalid OAM region
                0xFF00..=0xFF7F => todo!(), //Memory mapped ram
                0xFF80..= 0xFFFF => todo!(),//High ram
                //0xFFFF => todo!(), //Interrupts

            }
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
        pub fn get_graphics(&mut self) -> &[u8;8192]{
            self.memory_reference.get_graphics_memory()
        }
    }
}