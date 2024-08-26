pub mod vram{
    use std::ops::{Index, IndexMut};

    #[repr(packed)]
    pub struct Vram{
        pub block1:Block, //8000->87ff
        pub block2:Block,//8800 -> 9fff
        pub block3:Block, //9000-97ff
        pub tmap1:TileMap,
        pub tmap2:TileMap
    }
    impl Vram{

    }
    impl Index<u16> for Vram{
        type Output=u8;
    
        fn index(&self, index: u16) -> &Self::Output {
            todo!()
        }
    }
    impl IndexMut<u16> for Vram {
        fn index_mut(&mut self, index: u16) -> &mut Self::Output {
            todo!()
        }
    }
    pub struct TileMap{

    }
    pub struct Block{
        objects:[Vobj;128]
    }
    impl Index<u16> for Block{
        type Output=u8;
        fn index(&self, index: u16) -> &Self::Output {
            todo!()
        }
    }
    pub struct Vobj{ //2BPP Format!
        data:[u8;16]
    }
    impl Vobj{
        fn interleave_with_zeros(&mut self,a:u8) -> u16{
            let mut ret: u16 = a as u16; 
            ret = (ret ^ (ret<<4)) & 0x0f0f;//0000111100001111
            ret = (ret ^ (ret<<2)) & 0x3333;//0011001100110011
            (ret ^ (ret<<1)) & 0x5555//0101010101010101
        }
        fn interleave(&mut self,a:u8,b:u8)->u16{
            (self.interleave_with_zeros(b)<<1) | self.interleave_with_zeros(a)
        }
        fn get_tile(&mut self)->[u16;8]{
            let mut ret:[u16;8]=[0;8];
            for i in 0..7{
                ret[i] = Self::interleave(self,self.data[2*i],self.data[(2*i)+1]);
            }
            ret
        }
    }
}