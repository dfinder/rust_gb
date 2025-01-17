pub mod vram {

    use std::fmt::Debug;
    use log::debug;

    use crate::{memory::memory_wrapper::AsMemory, screen::screen::ColorID};

    
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
        pub fn debug(&mut self){
            for i in 0..=0x1FF {
                debug!("8{:X?}:{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}{:0>2X?}",i, 
                self.memory_map(i*16),
                self.memory_map(i*16+1),
                self.memory_map(i*16+2),
                self.memory_map(i*16+3),
                self.memory_map(i*16+4),
                self.memory_map(i*16+5),
                self.memory_map(i*16+6),
                self.memory_map(i*16+7),
                self.memory_map(i*16+8),
                self.memory_map(i*16+9),
                self.memory_map(i*16+10),
                self.memory_map(i*16+11),
                self.memory_map(i*16+12),
                self.memory_map(i*16+13),
                self.memory_map(i*16+14),
                self.memory_map(i*16+15))
            }
        }
    }
    impl AsMemory for Vram {
        fn memory_map(&mut self, addr: usize) -> u8 {
            match addr {
                0x0000..=0x07ff => self.block0.memory_map(addr),
                0x0800..=0x0fff => self.block1.memory_map(addr - 0x0800),
                0x1000..=0x17ff => self.block2.memory_map(addr - 0x1000),
                0x1800..=0x1bff => self.tmap1.memory_map(addr - 0x1800),
                0x1c00..=0x1fff => self.tmap2.memory_map(addr - 0x1c00),
                _ => unreachable!("owo"),
            }
        }

        fn memory_write(&mut self, addr: usize, val: u8) {
           
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

    #[derive(Debug)] 
    pub struct TileMap {
        pub tiles: [u8; 32*32], //I AM AN ORDERED STRUCTURE THAT REPRESENTS A MAPPING TO A BACKGROUND
    }
    impl TileMap {
        pub fn new() -> Self {
            return TileMap {
                tiles: [0;32* 32],
            };
        }
    }
    impl AsMemory for TileMap {
        fn memory_map(&mut self, addr: usize) -> u8 {
            return self.tiles[addr]
        }

        fn memory_write(&mut self, addr: usize, val: u8) {
            self.tiles[addr] = val
        }
    }

#[derive(Debug,Clone)] 
    pub struct Block {
        //I AM A BANK THAT HOLDS AN ENTIRE BACKGROUND
        pub objects: [Vobj; 128],
    }
    impl Block {
        pub fn new() -> Self {
            return Block {
                objects: [Self::vobj(); 128],
            };
        }
        fn vobj() -> Vobj {
            return [0; 16] ; //I AM A SET OF 8 PIXELS BEFORE PROCESSING
        }
        fn interleave_with_zeros(a: u8) -> u16 {
            let mut ret: u16 = a as u16;
            ret = (ret ^ (ret << 4)) & 0x0f0f; //0000111100001111
            ret = (ret ^ (ret << 2)) & 0x3333; //0011001100110011
            (ret ^ (ret << 1)) & 0x5555 //0101010101010101
        }
        fn interleave(a: u8, b: u8) -> u16 {
            (Block::interleave_with_zeros(b) << 1) | Block::interleave_with_zeros(a)
        }
        pub fn get_tile(vobj:Vobj) -> Tile {
            let mut ret: [u16; 8] = [0; 8];
            for i in 0..8 {
                ret[i] = Block::interleave(vobj[2 * i], vobj[(2 * i) + 1]);
            }
            return ret.map(|x| Block::color_palette(x));
        }
        pub fn get_tile_backwards(vobj:Vobj) -> Tile {
            let mut ret: [u16; 8] = [0; 8];
            for i in 0..8{
                ret[i] = Block::interleave(vobj[2 * i], vobj[(2 * i) + 1]);
            }
            let mut ret_tile = ret.map(Block::color_palette_reverse);
            ret_tile.reverse();
            return ret_tile;
        }
        pub fn color_palette(color: u16) -> [ColorID; 8] {
            let mut ret = [ColorID::Unset; 8];
            let mut loc_color = color;
            for i in 0..8 {
                ret[7 - i] = match loc_color & 0x03 {
                    0 => ColorID::Zero,
                    1 => ColorID::One,
                    2 => ColorID::Two,
                    3 => ColorID::Three,
                    _ => unreachable!(),
                };
                loc_color = loc_color >> 2;
            }
            ret
        }
        pub fn color_palette_reverse(color: u16) -> [ColorID; 8] {
            let mut ret = [ColorID::Unset; 8];
            let mut loc_color = color;
            for i in 0..8 {
                ret[7 - i] = match loc_color & 0x03 {
                    0 => ColorID::Zero,
                    1 => ColorID::Two,
                    2 => ColorID::One,
                    3 => ColorID::Three,
                    _ => unreachable!(),
                };
                loc_color = loc_color >> 2;
            }
            ret
        }
        
    }
    impl AsMemory for Block {
        fn memory_map(&mut self, addr: usize) -> u8 {
            self.objects[(addr >> 4) as usize][(addr % 16)as usize]
        }

        fn memory_write(&mut self, addr: usize, val: u8) {
            self.objects[(addr >> 4) as usize][(addr % 16) as usize]=val
        }
    }
    type Vobj= [u8; 16];
   
    #[test]
    fn test_interleave(){
        let a =0x3C;
        let b= 0x0;
        dbg!(Block::interleave(a, b));
        //assert_eq!(Block::interleave(a, b),0x2FF8);

        //assert_eq!(Block::interleave(0x42, 0x42),0x300C);


    }
    #[test]
    fn test_vobj(){
        let vobj:Vobj =[0xF0,0x00,0xF0,0x00,0xFC,0x00,0xFC,0x00,0xFC,0x00,0xFC,0x00,0xF3,0x00,0xF3,0x00] ;
        let background_palette = |x| match match x {
            ColorID::Zero => 0,
            ColorID::One => 3,
            ColorID::Two => 3,
            ColorID::Three => 3,
            ColorID::Unset => unreachable!(),
        } {
            0 =>"W",
            1 => "B",
            2 => "B",
            3 => "B",
            _ => unreachable!(),
        };
        for i in Block::get_tile(vobj){
            print!("{:?}\n",i.map(|x| background_palette(x)).join(""));
        }
    }
        
        
    }

