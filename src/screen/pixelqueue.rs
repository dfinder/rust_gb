pub mod pixelqueue{
    use crate::screen::{ppu::ppu::ColorID, vram::vram::Tile};

    #[derive(Copy,Clone)]
    pub struct FIFOPixel{
        pub color:ColorID,
        pub palette:bool,
        pub priority:bool
    }
    pub enum Stage{
        GetTile1,
        GetTile2,
        GetTileLow1,
        GetTileLow2,
        GetTileHigh1,
        GetTileHigh2,
        Sleep1,
        Sleep2,
        Push
    }
    pub struct TileWrapper{
        x:u8, 
        y:u8,
        tile:Tile,
    }
    pub struct PixelFIFO{
        pub pixel_list:[Option<FIFOPixel>;16],
        
    }
    impl PixelFIFO{
        pub fn new()->Self{
            return Self { pixel_list: [None;16]}
        }
        pub fn pop(&mut self)->Option<FIFOPixel>{
            self.pixel_list.rotate_left(1);
            let ret = self.pixel_list[15].clone();
            self.pixel_list[15]=None;
            return ret
        }
    }
   
}