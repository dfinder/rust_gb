pub mod vram{
    use crate::memory_wrapper::memory_wrapper::AsMemory;

    pub struct Vram{
        block1:Block, //8000->87ff
        block2:Block,//8800 -> 9fff
        block3:Block, //9000-97ff
        tmap1:TileMap,
        tmap2:TileMap
    }
    impl Vram{

    }
    impl AsMemory for Vram {
        fn memory_map(&mut self,addr:u16)->u8 {
            todo!()
        }
    
        fn memory_write(&mut self,addr:u16,val:u8){
            todo!()
        }
    }
    pub struct TileMap{

    }
    pub struct Block{
        objects:[Vobj;128]
    }
    impl Block{

    }
    pub struct Vobj{ //2BPP Format!
        data:[u8;16]
    }
    impl Vobj{
        fn interleave_with_zeros(self,a:u8) -> u16{
            let mut ret: u16 = a as u16; 
            ret = (ret ^ (ret<<4)) & 0x0f0f;//0000111100001111
            ret = (ret ^ (ret<<2)) & 0x3333;//0011001100110011
            (ret ^ (ret<<1)) & 0x5555//0101010101010101
        }
        fn interleave(self,a:u8,b:u8)->u16{
            (self.interleave_with_zeros(b)<<1) | self.interleave_with_zeros(a)
        }
        fn get_tile(self)->[u16;8]{
            let mut ret:[u16;8]=[0;8];
            for i in 0..7{
                ret[i] = Self::interleave(self,self.data[2*i],self.data[(2*i)+1]);
            }
            ret
        }
    }
}