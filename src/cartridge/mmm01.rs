pub mod mmm01 { //multicart 
    use crate::cartridge::mbc::mbc::Mbc;
    pub struct Mmm01 {
        rom: [u8; 0x8000],
        ram: [u8; 0x2000],
    }
    impl Mbc for Mmm01 {
        fn new(_cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
           todo!();
        }
        fn rom_read(&mut self, addr: usize) -> u8 {
            return self.rom[addr as usize];
        }

        fn rom_write(&mut self, _addr: usize, _val: u8) {
            ()
        }
        fn ram_read(&mut self, addr: usize) -> u8 {
            return self.ram[addr as usize];
        }
        fn ram_write(&mut self, addr: usize, val: u8) {
            self.ram[addr as usize] = val;
        }
    }
}
