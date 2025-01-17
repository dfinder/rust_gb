pub mod mbc0 {
    use crate::cartridge::mbc::mbc::Mbc;
    pub struct Mbc0 {
        rom: [u8; 0x8000],
        ram: [u8; 0x2000],
    }
    impl Mbc for Mbc0 {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
            let mut cart_rom: [u8; 0x8000] = [0; 0x8000];
            cart_rom.clone_from_slice(&cart.as_slice()[0..0x8000]);
            return Self {
                rom: cart_rom,
                ram: [0; 0x2000],
            };
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
