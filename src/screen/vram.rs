pub mod vram {

    use std::{thread, time::Duration};

    use log::info;

    use crate::{memory::memory_wrapper::AsMemory, screen::screen::ColorID};

    #[repr(packed)]
    pub struct Vram {
        pub block0: Block,  //8000->87ff
        pub block1: Block,  //8800->8fff
        pub block2: Block,  //9000->97ff
        pub tmap1: TileMap, //9800->9BFF
        pub tmap2: TileMap, //9C00->9FFF
    }
    pub type Tile = [[ColorID; 8]; 8];

    pub fn empty_tile() -> Tile {
        return [[ColorID::Unset; 8]; 8];
    }

    impl Vram {
        pub fn new() -> Self {
            return Vram {
                block0: Block::new(),
                block1: Block::new(),
                block2: Block::new(),
                tmap1: TileMap::new(),
                tmap2: TileMap::new(),
            };
        }
    }
    impl AsMemory for Vram {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000..=0x07ff => self.block0.memory_map(addr),
                0x0800..=0x0fff => self.block1.memory_map(addr - 0x0800),
                0x1000..=0x17ff => self.block2.memory_map(addr - 0x1000),
                0x1800..=0x1bff => self.tmap1.memory_map(addr - 0x1800),
                0x1c00..=0x1fff => self.tmap2.memory_map(addr - 0x1c00),
                _ => unreachable!("owo"),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            if val != 0 {
                info!("WE WRITE TO VRAM: {:X?}@{:X?}", val, addr);
            }
            match addr {
                0x0000..=0x07ff => self.block0.memory_write(addr, val),
                0x0800..=0x0fff => self.block1.memory_write(addr - 0x0800, val),
                0x1000..=0x17ff => self.block2.memory_write(addr - 0x1000, val),
                0x1800..=0x1bff => self.tmap1.memory_write(addr - 0x1800, val),
                0x1c00..=0x1fff => self.tmap2.memory_write(addr - 0x1c00, val),
                _ => unreachable!("owo"),
            }
        }
    }
    pub struct TileMap {
        pub tiles: [[u8; 32]; 32], //I AM AN ORDERED STRUCTURE THAT REPRESENTS A MAPPING TO A BACKGROUND
    }
    impl TileMap {
        pub fn new() -> Self {
            return TileMap {
                tiles: [[0; 32]; 32],
            };
        }
    }
    impl AsMemory for TileMap {
        fn memory_map(&mut self, addr: u16) -> u8 {
            return self.tiles[(addr >> 5) as usize][(addr % 32) as usize];
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            self.tiles[(addr >> 5) as usize][(addr % 32) as usize] = val
        }
    }
    pub struct Block {
        //I AM A BANK THAT HOLDS AN ENTIRE BACKGROUND
        pub objects: [Vobj; 128],
    }
    impl Block {
        pub fn new() -> Self {
            return Block {
                objects: [Vobj::new(); 128],
            };
        }
    }
    impl AsMemory for Block {
        fn memory_map(&mut self, addr: u16) -> u8 {
            self.objects[(addr >> 4) as usize].memory_map(addr % 16)
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            self.objects[(addr >> 4) as usize].memory_write(addr % 16, val)
        }
    }

    #[derive(Copy, Clone)]
    pub struct Vobj {
        //I AM A 8x8 SET OF PIXELS
        //2BPP Format!
        data: [u8; 16],
    }
    impl AsMemory for Vobj {
        fn memory_map(&mut self, addr: u16) -> u8 {
            self.data[addr as usize]
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            self.data[addr as usize] = val
        }
    }
    impl Vobj {
        fn new() -> Self {
            return Vobj { data: [0; 16] }; //I AM A SET OF 8 PIXELS BEFORE PROCESSING
        }
        fn interleave_with_zeros(a: u8) -> u16 {
            let mut ret: u16 = a as u16;
            ret = (ret ^ (ret << 4)) & 0x0f0f; //0000111100001111
            ret = (ret ^ (ret << 2)) & 0x3333; //0011001100110011
            (ret ^ (ret << 1)) & 0x5555 //0101010101010101
        }
        fn interleave(a: u8, b: u8) -> u16 {
            (Self::interleave_with_zeros(b) << 1) | Self::interleave_with_zeros(a)
        }
        pub fn get_tile(&self) -> Tile {
            let mut ret: [u16; 8] = [0; 8];
            for i in 0..7 {
                ret[i] = Self::interleave(self.data[2 * i], self.data[(2 * i) + 1]);
            }
            return ret.map(|x| Self::color_palette(x));
        }
        pub fn get_tile_backwards(&self) -> Tile {
            let mut ret: [u16; 8] = [0; 8];
            for i in 0..7 {
                ret[i] = Vobj::interleave(self.data[2 * i], self.data[(2 * i) + 1]);
            }
            let mut ret_tile = ret.map(|x| Self::color_palette_reverse(x));
            ret_tile.reverse();
            return ret_tile;
        }
        pub fn color_palette(color: u16) -> [ColorID; 8] {
            let mut ret = [ColorID::Unset; 8];
            let mut loc_color = color;
            for i in 0..8 {
                let pixel = color & 0x03;
                ret[7 - i] = match pixel {
                    0 => ColorID::Zero,
                    1 => ColorID::One,
                    2 => ColorID::Two,
                    3 => ColorID::Three,
                    _ => unreachable!(),
                };
                loc_color = loc_color >> 2
            }
            ret
        }
        pub fn color_palette_reverse(color: u16) -> [ColorID; 8] {
            let mut ret = [ColorID::Unset; 8];
            let mut loc_color = color;
            for i in 0..8 {
                let pixel = color & 0x03;
                ret[7 - i] = match pixel {
                    0 => ColorID::Zero,
                    1 => ColorID::Two,
                    2 => ColorID::One,
                    3 => ColorID::Three,
                    _ => unreachable!(),
                };
                loc_color = loc_color >> 2
            }
            ret
        }
    }
}
