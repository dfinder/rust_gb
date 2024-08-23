pub mod vscreen{
    use crate::memory_wrapper::memory_wrapper::{self, MemWrap};

    pub struct Vscreen{
        //vram:[u8;0x1800],
       // oam:[[u8;4];40],
    }
    pub enum PixelColor{

    }
    impl Vscreen{
        fn return_map(our_memory:MemWrap)->[[PixelColor;144];160]{
            our_memory.get_vram()
        }
    }
}