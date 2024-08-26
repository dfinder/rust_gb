pub mod vram;
pub mod mapped_io;
pub mod exram;
pub mod oam;
pub mod rom;
pub mod audio_controller;
pub mod video_controller;
pub mod memory_wrapper{
    use std::ops::{Index,IndexMut};

    use crate::cpu::cpu::CpuStruct;
    use super::{exram::exram::ExternalRam, mapped_io::mapped_io::MappedIO, oam::oam::OamStruct, vram::vram::Vram};

    //use crate::mapped_io;
    pub struct MemWrap{
        //memory_reference: MemoryStruct,
        rom:[u8;16384],
        vram:Vram,
        external_ram:ExternalRam, //Switchable bank
        work_ram:[u8;8192], // For CGB, switchable bank
        oam:OamStruct,
        mapped_io:MappedIO,
        high_ram:[u8;126],
    }
    impl Index<u16> for MemWrap{
        type Output= u8;

        fn index(&self, index: u16) -> &Self::Output {
            todo!()
        }
        
    }
    impl IndexMut<u16> for MemWrap{
        fn index_mut(&mut self, index: u16) -> &mut Self::Output {
            match index{
                0x0000..=0x7FFF => &mut self.rom[index as usize],//ROM
                0x8000..=0x9FFF => &mut self.vram[index-0x8000],//VRAM
                0xA000..=0xBFFF => &mut self.external_ram[index-0xa000],//External bus
                0xC000..=0xDFFF => &mut self.work_ram[(index-0xC000) as usize], //WRAM
                0xE000..=0xFDFF => todo!(), //ECHO
                0xFE00..=0xFE9F => &mut self.oam[index-0xFE00], // OAM
                0xFEA0..=0xFEFF => todo!(),//Invalid OAM region
                0xFF00..=0xFF7F => &mut self.mapped_io[index-0xFF00], //Memory mapped ram
                0xFF80..= 0xFFFE => &mut self.high_ram[0xFF80],//High ram
                0xFFFF => &mut self.mapped_io[0xFF], //Interrupts
            }
        }
    }
    impl MemWrap{
        pub fn new()->Self{
            Self { rom:[0;16384], vram: todo!(), external_ram: todo!(), work_ram: todo!(), oam: todo!(), mapped_io: todo!(), high_ram:[0;126] }
        }
        pub fn grab_memory_8(&mut self, addr:u16)->u8{
            CpuStruct::wait(2);
            self.memory_map(addr)
            //self.my_memory[addr as usize]
        }
        pub fn grab_memory_16(&mut self,addr:u16)->u16{ 
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
            CpuStruct::wait(4);
            (self[addr+1]as u16) <<8 + (self.memory_map(addr+1) as u16)
        }
        pub fn set_memory_8(&mut self, addr:u16, value:u8){
            self.memory_write(addr,value);
        }
        pub fn set_memory_16(&mut self, addr:u16, value:u16){
            self.memory_write(addr,(value>>8) as u8);
            self.memory_write(addr,(value % (1<<8)) as u8);
        }
        pub fn get_vram(&mut self)->&Vram{
            &self.vram
        }
    }
}