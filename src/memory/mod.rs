mod memory{
    public struct memory{
        my_memory:[u8;65535]
    }
    impl memory{
        fn init_memory()->memory{

        }
        fn grab_memory_8(&mut self, u16 addr)->{
            self.my_memory[addr]
        }
        fn grab_memory_16(&mut self,u16 addr)->{
            self.my_memory[addr+1] <<8 + self.my_memory[addr]
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
        }
        fn set_memory_8(&mut self, u16 addr, u8 value)->{
            self.my_memory[addr] = value
        }
        fn set_memory_16(&mut self, u16 addr, u16 value)->{
            self.my_memory[addr] = (value >> 8) as u8 
            self.my_memory[addr+1] = (value % (1<<8)) as u8
        }
    }
}