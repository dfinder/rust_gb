pub mod mbc7 { // accelerometer
    use crate::cartridge::mbc::mbc::Mbc;
    pub struct Mbc7 {
        rom: [u8; 0x8000],
        ram: [u8; 0x2000],
    }
    impl Mbc for Mbc7 {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
            let _ = cart;
            todo!()
        }
        fn rom_read(&mut self, addr: u16) -> u8 {
            return self.rom[addr as usize];
        }

        fn rom_write(&mut self, _addr: u16,_vall: u8) {
            ()
        }
        fn ram_read(&mut self, addr: u16) -> u8 {
            return self.ram[addr as usize];
        }
        fn ram_write(&mut self, addr: u16, val: u8) {
            self.ram[addr as usize] = val;
        }
    }
}
