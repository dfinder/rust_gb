pub mod mbc1 {
    use crate::cartridge::mbc::mbc::{ram_size, rom_size, Bank, Mbc};
    use std::cmp::max;
    pub struct Mbc1 {
        rom_bank_num: usize,
        ram_bank_num: usize,
        rom: Vec<Bank>,
        ram: Vec<Bank>,
        ram_enable: bool,
        bank_mode: bool,
    }
    impl Mbc for Mbc1 {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
            let rom: Vec<Bank> = vec![[0; 16384]; rom_size(cart[0x0148])];
            let ram: Vec<Bank> = vec![[0; 16384]; ram_size(cart[0x0149])];
            return Self {
                rom_bank_num: 0,
                ram_bank_num: 0,
                rom: rom,
                ram: ram,
                ram_enable: false,
                bank_mode: false,
            };
        }
        fn rom_read(&mut self, addr: u16) -> u8 {
            match addr {
                0..=0x3FFF => self.rom[0][addr as usize],
                0x4000..=0x7fff => self.rom[self.rom_bank_num][addr as usize],
                _ => unreachable!(),
            }
        }

        fn rom_write(&mut self, addr: u16, val: u8) {
            match addr {
                0..=0x1fff => self.ram_enable = val == 0xA,
                0x2000..=0x3fff => {
                    self.rom_bank_num =
                        ((self.ram_bank_num << 5) + (max(1, val & 0x1F) as usize)) % self.ram.len()
                }
                0x4000..=0x5fff => self.ram_bank_num = (val % 4) as usize,
                0x6000..=0x7fff => self.bank_mode = val % 2 == 1,
                _ => unreachable!(),
            }
        }
        fn ram_read(&mut self, addr: u16) -> u8 {
            //So this is the space A000->BFFF. We only go away from this if we turn on the bank mode.
            if !self.ram_enable {
                return 0xFF;
            } else {
                return self.ram[self.ram_bank_num * (self.bank_mode as usize)][addr as usize];
            }
        }
        fn ram_write(&mut self, addr: u16, val: u8) {
            if self.ram_enable {
                let ram_idx = self.ram_bank_num * (self.bank_mode as usize) % self.ram.len();
                self.ram[ram_idx][addr as usize] = val
            }
        }
    }
}
