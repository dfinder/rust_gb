pub mod vram;
pub mod mapped_io;
pub mod exram;
pub mod oam;
pub mod hram; 
pub mod rom;
pub mod memory_wrapper{
    use crate::cpu::cpu::CpuStruct;
    use super::{exram::exram::ExternalRam, hram::hram::HRam, mapped_io::mapped_io::MappedIO, oam::oam::OamStruct, vram::vram::Vram};

    //use crate::mapped_io;
    pub struct MemWrap{
        //memory_reference: MemoryStruct,
        rom:[u8;16384],
        vram:Vram,
        external_ram:ExternalRam, //Switchable bank
        work_ram:[u8;8192], // For CGB, switchable bank
        oam:OamStruct,
        mapped_io:MappedIO,
        high_ram:HRam,
    }
    pub trait AsMemory{
        fn memory_map(&mut self,addr:u16)->u8;
        fn memory_write(&mut self,addr:u16,val:u8);
    }
    impl AsMemory for MemWrap{
        fn memory_map(&mut self,addr:u16)->u8{
            match addr{
                0x0000..=0x7FFF => self.rom[addr as usize],//ROM
                0x8000..=0x9FFF => self.vram.memory_map(addr-0x8000),//VRAM
                0xA000..=0xBFFF => self.external_ram.memory_map(addr-0xa000),//External bus
                0xC000..=0xDFFF => self.work_ram[(addr-0xC000) as usize], //WRAM
                0xE000..=0xFDFF => todo!(), //ECHO
                0xFE00..=0xFE9F => self.oam.memory_map(addr-0xFE00), // OAM
                0xFEA0..=0xFEFF => todo!(),//Invalid OAM region
                0xFF00..=0xFF7F => self.mapped_io.memory_map(addr-0xFF00), //Memory mapped ram
                0xFF80..= 0xFFFF => self.high_ram.memory_map(0xFF80),//High ram
                0xFFFF => self.mapped_io.get_interrupt_enable_register(), //Interrupts

            }
        }
        fn memory_write(&mut self,addr:u16,val:u8){
            match addr{
                0x0000..=0x7FFF => self.rom[addr as usize] = val,//ROM
                0x8000..=0x9FFF => self.vram.memory_write(addr-0x8000,val),//VRAM
                0xA000..=0xBFFF => self.external_ram.memory_write(addr-0xa000,val),//External bus
                0xC000..=0xDFFF => self.rom[(addr-0xc000) as usize] = val, //WRAM
                0xE000..=0xFDFF => todo!(), //ECHO
                0xFE00..=0xFE9F => self.oam.memory_write(addr-0xFE00,val), // OAM
                0xFEA0..=0xFEFF => todo!(),//Invalid OAM region
                0xFF00..=0xFF7F => self.mapped_io.memory_write(addr-0xFF00,val), //Memory mapped ram
                0xFF80..= 0xFFFF => self.high_ram.memory_write(addr-0xFF80,val),//High ram
                0xFFFF => self.mapped_io.write_interrupt_enable_register(val), //Interrupts

            }
        }
    }
    impl MemWrap{
        pub fn new()->Self{
            Self { rom: todo!(), vram: todo!(), external_ram: todo!(), work_ram: todo!(), oam: todo!(), mapped_io: todo!(), high_ram: todo!() }
        }
        pub fn grab_memory_8(&mut self, addr:u16)->u8{
            CpuStruct::wait(2);
            self.memory_map(addr)
            //self.my_memory[addr as usize]
        }
        pub fn grab_memory_16(&mut self,addr:u16)->u16{ 
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
            CpuStruct::wait(4);
            (self.memory_map(addr+1) as u16) <<8 + (self.memory_map(addr+1) as u16)
        }
        pub fn set_memory_8(&mut self, addr:u16, value:u8){
            self.memory_write(addr,value);
        }
        pub fn set_memory_16(&mut self, addr:u16, value:u16){
            self.memory_write(addr,(value>>8) as u8);
            self.memory_write(addr,(value % (1<<8)) as u8);
        }
        pub fn get_vram(&mut self){
            self.vram
        }
    }
}