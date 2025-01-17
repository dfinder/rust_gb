pub mod mbc2 {
    use std::cmp::max;

    use crate::cartridge::mbc::mbc::{rom_size, Bank, Mbc};
    pub struct Mbc2 {
        rom_bank_num: usize,
        ram_bank_num: usize,
        rom: Vec<Bank>,
        ram: [u8; 512],
        ram_enable: bool,
        bank_mode: bool,
    }
    impl Mbc for Mbc2 {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
            let rom: Vec<Bank> = vec![[0; 16384]; rom_size(cart[0x0148])];
            return Self {
                rom_bank_num: 1,
                ram_bank_num: 0,
                rom: rom,
                ram: [0; 512],
                ram_enable: false,
                bank_mode: false,
            };
        }
        fn rom_read(&mut self, addr: usize) -> u8 {
            match addr {
                0..=0x3FFF => self.rom[0][addr as usize],
                0x4000..=0x7fff => self.rom[max(1, self.rom_bank_num % 16)][addr as usize],
                _ => unreachable!(),
            }
        }

        fn rom_write(&mut self, addr: usize, val: u8) {
            match addr {
                0..=0x1fff => {
                    if (addr & 0x10) == 0x10 {
                        self.rom_bank_num = max(1, val & 0x0f) as usize
                    } else {
                        self.ram_enable = val == 0xA;
                    }
                }
                0x2000..=0x3fff => {
                    self.rom_bank_num =
                        ((self.ram_bank_num << 5) + (max(1, val & 0x1F) as usize)) % self.ram.len()
                }
                0x4000..=0x5fff => self.ram_bank_num = (val % 4) as usize,
                0x6000..=0x7fff => self.bank_mode = val % 2 == 1,
                _ => unreachable!(),
            }
        }
        fn ram_read(&mut self, addr: usize) -> u8 {
            //So this is the space A000->BFFF. We only go away from this if we turn on the bank mode.
            if !self.ram_enable {
                return 0x0F;
            } else {
                return self.ram[(addr % 512) as usize] & 0x0F;
            }
        }
        fn ram_write(&mut self, addr: usize, val: u8) {
            if self.ram_enable {
                self.ram[(addr % 512) as usize] = val
            }
        }
    }
}
