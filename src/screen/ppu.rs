pub mod ppu {
    use std::{cell::RefCell, rc::Rc};

    use crate::screen::{oam::oam::OamStruct, video_controller::video_controller::VideoController, vram::vram::Vram};


    pub struct Ppu {
        //This gets to be _fun_
        virtual_internal_screen: [[PixelColor; 256]; 256],
        virtual_bounded_screen: [[PixelColor; 160]; 144],
        pub vram: Vram,
        pub oam: OamStruct,
        video_controller: Rc<RefCell<VideoController>>, //vram:[u8;0x1800],
                                           //oam:[[u8;4];40],
    }
    impl Ppu {
        pub fn new(vc: Rc<RefCell<VideoController>>)->Self{
            return Self { virtual_internal_screen: [[PixelColor::Transparent;256];256], virtual_bounded_screen: [[PixelColor::Transparent;160];144], vram: Vram::new(), oam: OamStruct::new(), video_controller: vc }
        }
        pub fn get_screen(&self) -> [[PixelColor; 160]; 144] {
            return self.virtual_bounded_screen;
        }
    }
    #[derive(Clone, Copy)]
    pub enum PixelColor {
        White,
        LightGrey,
        DarkGrey,
        Black,
        Transparent,
    }

    impl Ppu {

        fn draw_dot(&mut self) {
            fn process_background() {

            }
            let vc=self.video_controller.borrow();
            let mode = vc.stat % 4;
            if mode == 0 {}
            if vc.lcdc % 2 == 1 {
                process_background()
                //Background is on, check background first.
            }
            if (vc.lcdc >> 1) % 2 == 1 {
                //Obj is on

                if (vc.lcdc >> 2) % 2 == 1 {
                    //Obj is on
                }
            }
        }
        fn return_map(&self) -> [[PixelColor; 160]; 144] {
            return self.virtual_bounded_screen;
        }
    }
}
