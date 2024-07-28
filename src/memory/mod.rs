pub mod memory{
    
    use std::fs::File;
    use std::io::Read;

    use crate::cpu;
    pub struct MemoryStruct{
        my_memory:[u8;65535]
    }
    impl MemoryStruct{
        pub fn init_memory()->Self{
            let mut my_memory=[0 as u8;65535];
            let filename = "./DMG_ROM.bin";
            let mut f = File::open(&filename).expect("no file found"); //Len = 256
            f.read(&mut my_memory).expect("buffer overflow");
            return Self {
                 my_memory:my_memory
             }
        }
        pub fn grab_memory_8(&mut self, addr:u16)->u8{
            cpu::cpu::wait(2);
            self.my_memory[addr as usize]
        }
        pub fn grab_memory_16(&mut self,addr:u16)->u16{

            (self.my_memory[(addr+1) as usize] as u16) <<8 + (self.my_memory[addr as usize] as u16)
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
        }
        pub fn set_memory_8(&mut self, addr:u16, value:u8){
            self.my_memory[addr as usize] = value
        }
        pub fn set_memory_16(&mut self, addr:u16, value:u16){
            self.my_memory[addr as usize] = (value >> 8) as u8 ;
            self.my_memory[(addr+1) as usize] = (value % (1<<8)) as u8;
        }
    }
}