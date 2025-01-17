pub mod mbc;
mod mbc0;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod mbc6;
mod mbc7;
mod mmm01;
pub mod cartridge {
    use std::{fs::File, io::Read};


    use crate::memory::memory_wrapper::AsMemory;

    use super::{
        mbc::mbc::Mbc, mbc0::mbc0::Mbc0, mbc1::mbc1::Mbc1, mbc2::mbc2::Mbc2, mbc3::mbc3::Mbc3,
        mbc5::mbc5::Mbc5, mbc6::mbc6::Mbc6, mbc7::mbc7::Mbc7, mmm01::mmm01::Mmm01,
    };
    pub enum Cartridge {
        Mbc0(Mbc0),
        Mbc1(Mbc1),
        Mbc2(Mbc2),
        Mbc3(Mbc3),
        Mbc5(Mbc5),
        Mbc6(Mbc6),
        Mbc7(Mbc7),
        Mmm01(Mmm01),
    }
    impl Cartridge {
        pub fn new(mut cart: File) -> Self {
            let mut cart_contents: Vec<u8> = Vec::<u8>::new();
            cart.read_to_end(&mut cart_contents)
                .expect("cart not found");
            //println!("CARTRIDGE FOUND {:?}",cart_contents[0x0147] );
            return match cart_contents[0x0147] {
                0x00 => Cartridge::Mbc0(Mbc0::new(cart_contents)),
                0x01 => Cartridge::Mbc1(Mbc1::new(cart_contents)),
                0x02 => Cartridge::Mbc1(Mbc1::new(cart_contents)),
                0x03 => Cartridge::Mbc1(Mbc1::new(cart_contents)),
                0x05 => Cartridge::Mbc2(Mbc2::new(cart_contents)),
                0x06 => Cartridge::Mbc2(Mbc2::new(cart_contents)),
                0x08 => unreachable!(),
                0x09 => unreachable!(),
                0x0b => Cartridge::Mmm01(Mmm01::new(cart_contents)),
                0x0c => Cartridge::Mmm01(Mmm01::new(cart_contents)),
                0x0d => Cartridge::Mmm01(Mmm01::new(cart_contents)),
                0x0f => Cartridge::Mbc3(Mbc3::new(cart_contents)),
                0x10 => Cartridge::Mbc3(Mbc3::new(cart_contents)),
                0x11 => Cartridge::Mbc3(Mbc3::new(cart_contents)),
                0x12 => Cartridge::Mbc3(Mbc3::new(cart_contents)),
                0x13 => Cartridge::Mbc3(Mbc3::new(cart_contents)),
                0x19 => Cartridge::Mbc5(Mbc5::new(cart_contents)),
                0x1a => Cartridge::Mbc5(Mbc5::new(cart_contents)),
                0x1b => Cartridge::Mbc5(Mbc5::new(cart_contents)),
                0x1c => Cartridge::Mbc5(Mbc5::new(cart_contents)),
                0x1d => Cartridge::Mbc5(Mbc5::new(cart_contents)),
                0x1e => Cartridge::Mbc5(Mbc5::new(cart_contents)),
                0x20 => Cartridge::Mbc6(Mbc6::new(cart_contents)),
                0x22 => Cartridge::Mbc7(Mbc7::new(cart_contents)),
                0xfc => todo!(), //poccam(),,
                0xfd => todo!(), //bandai tama5
                0xfe => todo!(), //huc3
                0xff => todo!(), //huc1
                _ => unreachable!(),
            };
        }   
    }
    impl AsMemory for Cartridge {
        fn memory_map(&mut self, addr: usize) -> u8 {
            if addr >= 0xA000 {
                return match self {
                    Cartridge::Mbc0(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mbc1(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mbc2(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mbc3(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mbc5(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mbc6(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mbc7(mem) => mem.ram_read(addr - 0xA000),
                    Cartridge::Mmm01(mem) => mem.ram_read(addr - 0xA000),
                };
            } else {
                return match self {
                    Cartridge::Mbc0(mem) => mem.rom_read(addr),
                    Cartridge::Mbc1(mem) => mem.rom_read(addr),
                    Cartridge::Mbc2(mem) => mem.rom_read(addr),
                    Cartridge::Mbc3(mem) => mem.rom_read(addr),
                    Cartridge::Mbc5(mem) => mem.rom_read(addr),
                    Cartridge::Mbc6(mem) => mem.rom_read(addr),
                    Cartridge::Mbc7(mem) => mem.rom_read(addr),
                    Cartridge::Mmm01(mem) => mem.rom_read(addr),
                };
            }
        }

        fn memory_write(&mut self, addr: usize, val: u8) {
            if addr > 0xA000 {
                return match self {
                    Cartridge::Mbc0(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mbc1(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mbc2(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mbc3(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mbc5(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mbc6(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mbc7(mem) => mem.ram_write(addr - 0xA000, val),
                    Cartridge::Mmm01(mem) => mem.ram_write(addr - 0xA000, val),
                };
            } else {
                return match self {
                    Cartridge::Mbc0(mem) => mem.rom_write(addr, val),
                    Cartridge::Mbc1(mem) => mem.rom_write(addr, val),
                    Cartridge::Mbc2(mem) => mem.rom_write(addr, val),
                    Cartridge::Mbc3(mem) => mem.rom_write(addr, val),
                    Cartridge::Mbc5(mem) => mem.rom_write(addr, val),
                    Cartridge::Mbc6(mem) => mem.rom_write(addr, val),
                    Cartridge::Mbc7(mem) => mem.rom_write(addr, val),
                    Cartridge::Mmm01(mem) => mem.rom_write(addr, val),
                };
            }
        }
    }
}
