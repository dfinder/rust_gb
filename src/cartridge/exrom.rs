pub mod exrom {
    use std::{fs::File, io::Bytes};

    use crate::memory_wrapper::memory_wrapper::AsMemory;
    type Bank = [u8; 0x4000];
    pub struct ExROM {
        bank_num: u8,
        bank_size: u8,
        rom: Bank,
        banks: Vec<Bank>,
    }
    impl ExROM {
        pub fn new(contents: Vec<u8>) -> Self {
            let base_rom: u8 = 2 << contents[0x0148];
            //bank_size = 16*1024;
            let mut banks = Vec::<Bank>::new();
            for _i in 0..base_rom {
                banks.push([0; 0x4000]);
            }
            return Self {
                bank_num: 0x0001,
                bank_size: base_rom - 1,
                rom: [0; 0x4000],
                banks: banks,
            };
        }
    }

    impl AsMemory for ExROM {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000..=0x3FFF => todo!(),
                0x4000..=0x7FFF => todo!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000..=0x3FFF => todo!(),
            }
        }
    }
}
