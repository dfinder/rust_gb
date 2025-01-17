pub mod mbc5 {

    use crate::cartridge::mbc::mbc::{ram_size, rom_size, Bank, Mbc};
    pub struct Mbc5 {
        rom_bank_num: usize,
        ram_bank_num: usize,
        rom: Vec<Bank>,
        ram: Vec<Bank>,
        ram_enable: bool,
    }
    impl Mbc for Mbc5 {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
            let ram: Vec<Bank> = vec![[0; 16384]; ram_size(cart[0x0147])];
            let rom: Vec<Bank> = vec![[0; 16384]; rom_size(cart[0x0148])];
            return Self {
                rom_bank_num: 1,
                ram_bank_num: 0,
                rom: rom,
                ram: ram,
                ram_enable: false,
            };
        }
        fn rom_read(&mut self, addr: usize) -> u8 {
            match addr {
                0..=0x3FFF => self.rom[0][addr as usize],
                0x4000..=0x7fff => self.rom[self.rom_bank_num % 16][addr as usize],
                _ => unreachable!(),
            }
        }

        fn rom_write(&mut self, addr: usize, val: u8) {
            //Note there's something about rumble here.
            match addr {
                0..=0x1fff => self.ram_enable = val == 0x0A,
                0x2000..=0x2fff => self.rom_bank_num = (self.rom_bank_num & 0x1ff) + (val as usize), //Write 8 bits, preserving bit 9
                0x3000..=0x3fff => {
                    self.rom_bank_num = ((val & 0x01) << 8 + (self.rom_bank_num & 0x0ff)) as usize
                } //Write 9th bit
                0x4000..=0x7fff => self.ram_bank_num = (val % 4) as usize,
                _ => unreachable!(),
            }
        }
        fn ram_read(&mut self, addr: usize) -> u8 {
            //So this is the space A000->BFFF. We only go away from this if we turn on the bank mode.
            if !self.ram_enable {
                return 0x0F;
            } else {
                return self.ram[self.ram_bank_num][addr as usize];
            }
        }
        fn ram_write(&mut self, addr: usize, val: u8) {
            if self.ram_enable {
                self.ram[self.ram_bank_num][addr as usize] = val
            }
        }
    }
}
