pub mod ppu{
    use crate::memory_wrapper::{memory_wrapper::{self, MemWrap}, oam::oam::OamStruct,vram::vram::Vram};
    pub struct VideoController{
        pub lcdc: u8,//LCD control. Tell sus 
        pub stat: u8, 
        pub scy: u8, //
        pub scx: u8,
        pub ly: u8 ,
        pub lyc: u8,
        pub dma: u8,
        pub bgp: u8, 
        pub obp0: u8,
        pub obp1: u8,
        pub wy: u8,
        pub wx: u8,
    }

    pub struct Ppu{ //This gets to be _fun_
        virtual_internal_screen: [[PixelColor;256];256],
        virtual_bounded_screen: [[PixelColor;160];144],
        vram:Vram,
        oam:OamStruct,
        video_controller:VideoController
        //vram:[u8;0x1800],
       // oam:[[u8;4];40],
    }
    #[derive(Clone,Copy)]
    pub enum PixelColor{
        White,
        LightGrey,
        DarkGrey,
        Black,
        Transparent
    }

    impl Ppu{
        fn process_background(&mut self){

        }
        fn draw_dot(&mut self){
            let mode = self.video_controller.stat %4;
            if mode ==0{

            }
            if self.video_controller.lcdc %2 == 1 {
                self.process_background()
                //Background is on, check background first.
            }
            if (self.video_controller.lcdc>>1) %2 == 1 {
                //Obj is on

            if (self.video_controller.lcdc>>2) %2 == 1 {
                //Obj is on
            }
            }


            
        }
        fn return_map(&self)->[[PixelColor;160];144]{
            return self.virtual_bounded_screen;
        }
    }
}