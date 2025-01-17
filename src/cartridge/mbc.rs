pub mod mbc {
    pub type Bank = [u8; 0x4000];
    pub trait Mbc {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized;
        fn rom_read(&mut self, addr: usize) -> u8;
        fn rom_write(&mut self, addr: usize, val: u8);
        fn ram_read(&mut self, addr: usize) -> u8;
        fn ram_write(&mut self, addr: usize, val: u8);
    }
    pub fn rom_size(val: u8) -> usize {
        match val {
            0 => 2,
            1 => 4,
            2 => 8,
            3 => 16,
            4 => 32,
            5 => 64,
            6 => 128,
            7 => 256,
            8 => 512,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => unreachable!(),
        }
    }

    pub fn ram_size(val: u8) -> usize {
        match val {
            0 => 0,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => unreachable!(),
        }
    }
}
