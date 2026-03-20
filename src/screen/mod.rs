//
//use crate::Surface
pub mod oam;
pub mod pixelqueue;
pub mod video_controller;
pub mod vram;
pub mod screen {

    fn bit(val: u8, n: u8) -> bool {
        (val & (1 << n)) != 0
    }
    use log::{debug, info};
    use sdl2::{pixels::Color, rect::Point, render::Canvas, video::Window};

    use crate::{
        cpu::cpu::Interrupt, memory::memory_wrapper::AsMemory, screen::{
            oam::oam::OamStruct, video_controller::{self, video_controller::VideoController},
            vram::vram::Vram,
        }
    };

    use super::vram::vram::Block;

    type VirtualBoundedScreen = [[GBColor; 160]; 144];
    pub struct Screen {
        virtual_unrendered_screen: VirtualBoundedScreen,
        virtual_rendered_screen: VirtualBoundedScreen,
        pub vram: Vram,
        pub oam: OamStruct,
        canvas: Canvas<Window>,
        dots: u16,
        objs: ([Option<u8>; 10], u8),
        pub vc: VideoController, //vram:[u8;0x1800],
                                 //oam:[[u8;4];40],
    }
    #[derive(Clone, Copy, Hash, PartialEq, std::cmp::Eq, Debug)]
    pub enum GBColor {
        White,
        LightGrey,
        DarkGrey,
        Black,
        Transparent,
        Baka
    }
    #[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
    pub enum ColorID {
        Unset,
        Zero,
        One,
        Two,
        Three,
    }
    /**pub struct PixelFetcher{
        mode_3_penalty:u8,
        background_fifo:PixelFIFO,
        obj_fifo:PixelFIFO,
        objs:([Option<u8>;10],u8),
        current_pixel:u8,
        stage:Stage,

    }**/
    impl Screen {
        pub fn new(canvas: Canvas<Window>) -> Self {
            return Self {
                virtual_unrendered_screen: [[GBColor::Transparent; 160]; 144],
                virtual_rendered_screen: [[GBColor::White; 160]; 144],
                vram: Vram::new(),
                oam: OamStruct::new(),
                dots: 0, //0..456
                canvas,
                vc: VideoController {
                    bgp: 0,
                    dma: 0,
                    lcdc: 0,
                    ly: 0,
                    lyc: 0,
                    obp0: 0,
                    obp1: 0,
                    scx: 0,
                    scy: 0,
                    wx: 0,
                    wy: 0,
                    stat: 0x02,
                },
                objs: ([None; 10], 0),
            };
        }
        pub fn on_clock(&mut self)->(Option<Interrupt>, Option<Interrupt>){ //Move 0, Mode 1?
            if !bit(self.vc.lcdc,7){
                return (None,None);
            }
            //On clock, we draw a single dot. 
            let mut  ret =  (None,None);
            self.dots = (self.dots+1)%456;
            if self.dots == 0{
                self.vc.ly = (self.vc.ly+1) % 154;
                if self.vc.ly < 144{
                    self.vc.stat = (self.vc.stat & 0xFE) | 0x02; //Move to 2
                    if (self.vc.stat & 0x20) != 0 {
                        ret.1 = Some(Interrupt::LCDC);
                    }
                    //Fetch Line
                    self.virtual_unrendered_screen[self.vc.ly as usize] = self.fetch_line();
                }
                else if self.vc.ly == 144{
                    self.vc.stat = self.vc.stat | 0x01; //Move from 0 to 1

                    ret.0 = Some(Interrupt::VBlank);
                    if self.vc.stat & 0x10 != 0{
                        ret.1 = Some(Interrupt::LCDC); //Vblank stat 
                    }

                }
                if self.vc.lyc == self.vc.ly{
                    self.vc.stat = self.vc.stat | 0x04; 
                    if self.vc.stat & 0x40 != 0 {
                        ret.1 = Some(Interrupt::LCDC); // LYC=LY coincidence
                    }
                } 
            }
            if self.dots == 80{ //Move from 2 to 3
                self.vc.stat = self.vc.stat | 0x01;
                    //Draw line
            }
            if self.dots == 172 && self.vc.ly<144{ //Move from 3 to 0
                self.vc.stat = self.vc.stat & 0xFC;
                    self.virtual_rendered_screen[self.vc.ly as usize] = self.virtual_unrendered_screen[self.vc.ly as usize];

                //TRIGGER HBLANK
                if self.vc.stat & 0x08 !=0 {
                    ret.1 = Some(Interrupt::LCDC);
                }
            }
            ret 

        }
        pub fn fetch_line(&mut self) -> [GBColor; 160]{
            
            let ret = [GBColor::Transparent;160];
            let bg_mapping = |id:u8| match id{
                0 => GBColor::White,
                1 => GBColor::LightGrey,
                2 => GBColor::DarkGrey,
                3 => GBColor::Black,
                _ => GBColor::Baka
            };
            let mapping = |id:ColorID| match id {
                ColorID::Unset => GBColor::Baka,
                ColorID::Zero => bg_mapping(self.vc.bgp% 4),
                ColorID::One => bg_mapping((self.vc.bgp>>2)%4),
                ColorID::Two => bg_mapping((self.vc.bgp>>4)%4),
                ColorID::Three => bg_mapping((self.vc.bgp>>6)%4),
            };
            //Grab background
            let mut background = self.vram.fetch_line(bit(self.vc.lcdc,3), bit(self.vc.lcdc,4), self.vc.ly.wrapping_add(self.vc.scy), self.vc.scx).map(mapping);
            //let background_line = background[self.vc.scx..self.vc.scx+160];
            //Grab Window
            if bit(self.vc.lcdc,5) && self.vc.wy>= self.vc.ly {
                let window = self.vram.fetch_line(bit(self.vc.lcdc,6), bit(self.vc.lcdc,4), self.vc.ly.wrapping_sub(self.vc.wy),0).map(mapping);
                for i in (self.vc.wx as i32 - 7)..160i32{
                    if i>=0{
                        background[i as usize] = window[(i-(self.vc.wx as i32-7)) as usize]
                    }
                }
            }
            //Grab Sprites
            return background;
        }
        pub fn get_screen(&self) -> VirtualBoundedScreen {
            //println!("We get the screen");
            return self.virtual_rendered_screen;
        }
        pub fn read_vram(&mut self,addr:usize) -> u8{
            if self.vc.stat %4!=3  || !bit(self.vc.lcdc,7){
                return self.vram.memory_map(addr)
            }
            return 0xff
        }
        pub fn read_oam(&mut self,addr:usize) -> u8{
            if self.vc.stat %4<3 || !bit(self.vc.lcdc,7){
                return self.oam.memory_map(addr)
            }
            return 0xff
        }
        pub fn write_oam(&mut self,addr:usize,value:u8){

            if self.vc.stat %4!=3 || !bit(self.vc.lcdc,7){
                println!("{:x}",addr);
                self.vram.memory_write(addr,value)
            }
        }
        pub fn write_vram(&mut self,addr:usize,value:u8){
            if self.vc.stat %4<3 || !bit(self.vc.lcdc,7){
                self.oam.memory_write(addr,value)
            }
        }
    }
}
