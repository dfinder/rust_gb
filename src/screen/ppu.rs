pub mod ppu {
    use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};


    use crate::{
        interrupt::interrupt::Interrupt,
        memory_wrapper::memory_wrapper::AsMemory,
        screen::{
            oam::oam::{Oam, OamStruct},
            vram::vram::Vram,
        },
    };
    #[derive(Clone, Copy)]
    pub struct VideoController {
        pub lcdc: u8, //LCD Control
        pub stat: u8, //Interrupts
        pub scy: u8,  //Background viewport Y
        pub scx: u8,  //Background viewport X
        pub ly: u8,   //Line of drawing
        pub lyc: u8,  //Line to compare
        pub dma: u8,  //DMA!
        pub bgp: u8,  //Background Palette Data
        pub obp0: u8, //Oboject palette 1
        pub obp1: u8, //Object palette 2
        pub wy: u8,   //Window Y
        pub wx: u8,   //Window X
    }
    impl AsMemory for VideoController {
        //Remember
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0 => self.lcdc, //LCD control register, controls which banks are used, whether windows/background is used, etc. 
                1 => self.stat, //Interrupts
                2 => self.scy, //Background viewport Y
                3 => self.scx,  //Background viewport X
                4 => self.ly,   //Line of drawing
                5 => self.lyc,  //Line to compare
                6 => self.dma,  //DMA!
                7 => self.bgp,  //Background Pallete Data
                8 => self.obp0, //Oboject palette 1
                9 => self.obp1, //Object palette 2
                0xa => self.wy, //Window Y
                0xb => self.wx, //Window X
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0 => self.lcdc = val,
                1 => self.stat = val&0xFB,
                2 => self.scy = val,    //Background Viewport Y
                3 => self.scx = val,  //Background viewport X
                4 => (),   //Line of drawing, is READ ONLY
                5 => self.lyc = val,  //Line to compare
                6 => self.dma = val,  //DMA! Controls a fun bypass between RAM and VRAM
                7 => self.bgp = val,  //Background Pallette Data
                8 => self.obp0 = val, //Object palette 1
                9 => self.obp1 = val, //Object palette 2
                0xa => self.wy = val, //Window Y
                0xb => self.wx = val, //Window X
                _ => unreachable!(),
            }
        }
    }
    type VirtualInternalScreen = [[PaletteColor; 256]; 256];
    type ObjScreen = [[PaletteColor; 160]; 144];
    type VirtualBoundedScreen = [[GBColor; 160]; 144];
    struct  PreRenderedScreen {
        background:VirtualInternalScreen,window:VirtualInternalScreen,objects:ObjScreen
    }
    #[derive(Copy,Clone)]
    struct FIFOPixel{
        color:PaletteColor,
        palette:bool,
        priority:bool
    }
    pub struct Ppu {
        //This gets to be _fun_
        virtual_screen_layers: PreRenderedScreen,
        virtual_unrendered_screen: VirtualBoundedScreen,
        virtual_rendered_screen: VirtualBoundedScreen,
        pub vram: Vram,
        pub oam: OamStruct,
        dots: u16,
        mode_3_penalty: u16,
        background_fifo: [Option<FIFOPixel>;16],

        obj_fifo: [Option<FIFOPixel>;16],
        video_controller: Rc<RefCell<VideoController>>, //vram:[u8;0x1800],
                                                        //oam:[[u8;4];40],
    }
    #[derive(Clone, Copy,Hash,PartialEq,std::cmp::Eq)]
    pub enum GBColor {
        White,
        LightGrey,
        DarkGrey,
        Black,
        Transparent,
    }
    #[derive(Copy,Clone)]
    #[derive(Eq, Hash, PartialEq)]
    pub enum PaletteColor {
        Unset,
        Zero,
        One,
        Two,
        Three
    }
    impl Ppu {
        pub fn new(vc: Rc<RefCell<VideoController>>) -> Self {
            return Self {
                virtual_unrendered_screen: [[GBColor::Transparent; 160]; 144],
                virtual_rendered_screen: [[GBColor::Transparent; 160];144],
                virtual_screen_layers:PreRenderedScreen{ background:[[PaletteColor::Unset;256];256], window:[[PaletteColor::Unset;256];256], objects: [[PaletteColor::Unset;160];144]},
                vram: Vram::new(),
                oam: OamStruct::new(),
                dots: 0, //0..456
                video_controller: vc,
                mode_3_penalty: 0,
                background_fifo:[None;16],
                obj_fifo:[None;16]
               
            };
        }
        pub fn get_screen(&self) -> VirtualBoundedScreen {
            return self.virtual_rendered_screen;
        }
        pub fn hblank(&mut self) {}
        pub fn vblank(&mut self) {}
        pub fn init_mode_3(&mut self) {
            //let mut ret= [[PaletteColor::Unset; 167]; 144+16+16];
            let vc;
            {
                vc = self.video_controller.borrow().clone();
            }
            let window_enabled = vc.lcdc >> 5 % 2 == 1;
            if vc.lcdc % 2 == 1 {
                self.generate_background();
                //for y in 0..182 {
                //    for x in 7..151 {
                //        self.virtual_prerendered_screen.background[y][x] =
                //            background[(vc.scy as usize + y) % 256][vc.scx as usize + x % 256]
                //    }
                //}
                if window_enabled {
                    self.generate_window();
                    //for y in 0..182 - vc.wy {
                    //    for x in 0..151 - vc.wx {
                    //        if x - 7 + vc.wx > 0 {
                    //            ret[(vc.wy + y) as usize][(x - 7 + vc.wx) as usize] =
                    //                window[y as usize][x as usize]
                    //        }
                    //    }
                    //}
                }
                //Background is on, check background first.
                 // for i in 16..160{
                //     for j in 7..167{
                //         self.virtual_unrendered_screen[j-7][i-16]=ret[j][i]
                //     }
                //  }
            }
        }
          
        pub fn oam_scan(&mut self){
            let vc;
            {
                vc = self.video_controller.borrow().clone();
            }
            if (vc.lcdc >> 1) % 2 == 1 {
                let mut objects: [Option<Oam>; 10] = [None; 10];
                let mut count = 0;
                let big = (vc.lcdc >> 2) % 2 == 1;
                for oam in self.oam.oam_list{
                    if count==10{
                        break
                    }
                    if (oam.ypos..oam.ypos+8+(8*(big as u8))).contains(&vc.ly){
                        objects[count]=Some(oam);
                        count+=1;
                    }
                }
                //Obj is on
                if !big{
                    //Obj is sized with 8x8
                } else { //Obj is sized with 8x16

                }
            }
                
        }
        fn draw_pixel(&mut self){

        }
        pub fn on_ppu(&mut self) -> (Option<Interrupt>,Option<Interrupt>) {
            let mut interrupt:(Option<Interrupt>,Option<Interrupt>) = (None,None);
            let mut run_mode_2: bool = false;
            let mut run_mode_3: bool = false;
            let ly: u8;
            {
                let mut vc = self.video_controller.borrow_mut();
                ly = vc.ly;

                self.dots += 1; //Increase the dot
                if self.dots==79 && ly<144{
                    run_mode_2=true
                }
                if self.dots == 80 && ly < 144 {
                    vc.stat += 1; //2->3    
                    run_mode_3=true
                }
            }
            {
                if run_mode_2 {
                    self.oam_scan(); //This is when we 
                }
                if run_mode_3 {
                    self.init_mode_3(); //This is when we 
                }
  
                {
                    let mut vc = self.video_controller.borrow_mut();
                    if self.dots > 456 {
                        //On row adjustment
                        self.dots = 0;
                        vc.ly += 1;
                        if vc.ly == vc.lyc { //Manage LCY 
                            vc.stat |= 0x04; //Set bit 2 to true
                            if vc.stat & 0x40 > 0 {
                                //Test if LCDC bit 6 is active
                                interrupt.0= Some(Interrupt::LCDC)
                            }
                        } else {
                            vc.stat &= 0xFB //Turn off bit 2, resetting us to mode 1
                        }
                        if vc.ly < 144 {//ENTER 
                            //We go from 0->2
                            vc.stat += 2;
                            if vc.stat & 0x20 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt.0 = Some(Interrupt::LCDC)
                            }
                        }
                        if vc.ly == 144 {
                            //We're in 0->1
                            vc.stat += 1;
                            self.virtual_rendered_screen=self.virtual_unrendered_screen;
                            self.virtual_unrendered_screen= [[GBColor::Transparent; 160]; 144]; //Reset the internal screen
                            interrupt.1 = Some(Interrupt::VBlank) //Remember, VBlank is a separate  
                        }
                        if vc.ly > 153 {
                            //1->2
                            vc.ly = 0;
                            vc.stat += 1;
                            if vc.stat & 0x20 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt.0 = Some(Interrupt::LCDC)
                            }
                        }
                    }
                }
                if self.mode()==3{
                    self.draw_pixel();
                }
                return interrupt;
            }
        }
        fn pre_bg_palette(color:u16)->[PaletteColor;8]{
            let mut ret = [PaletteColor::Unset; 8];
            let mut loc_color = color;
            for i in 0..8 {
                let pixel = color & 0x03;
                ret[7 - i] = match pixel {
                    0 => PaletteColor::Zero,
                    1 => PaletteColor::One,
                    2 => PaletteColor::Two,
                    3 => PaletteColor::Three,
                    _ => unreachable!(),
                };
                loc_color = loc_color >> 2
            }
            ret
        }
        fn generate_window(&mut self){
            //let pre_textured :[[u8;256];256]=[[0;256];256];
            let lcdc: u8;
            {
                lcdc = self.video_controller.borrow().lcdc;
            }
            if lcdc >> 7 == 0 {
                self.virtual_screen_layers.window=[[PaletteColor::Unset; 256]; 256];
            }
            let bg_tile_map: &[[u8; 32]; 32];
            if lcdc & 0x40 > 0 {
                //
                bg_tile_map = &self.vram.tmap2.tiles;
            } else {
                bg_tile_map = &self.vram.tmap1.tiles;
            }
            let pixel_map = |x: u16| Ppu::pre_bg_palette( x);
            let tile_map = |x: u8| {
                match x {
                    0..=0x7f => {
                        if lcdc & 0x10 > 0 {
                            self.vram.block0.objects[x as usize]
                        } else {
                            self.vram.block2.objects[x as usize]
                        }
                    }
                    0x80..=0xFF => self.vram.block1.objects[(x - 128) as usize],
                }
                .get_tile()
                .map(pixel_map)
            };
            let bg_tiles = bg_tile_map.map(|x| x.map(|y| tile_map(y)));
            for x in 0..32 {
                for y in 0..32 {
                    for x_p in 0..8 {
                        for y_p in 0..8 {
                            self.virtual_screen_layers.window[y * 8 + y_p][x * 8 + x_p] = bg_tiles[y][x][y_p][x_p]
                        }
                    }
                }
            }
        }
        fn generate_background(&mut self) {
            //let pre_textured :[[u8;256];256]=[[0;256];256];
            let lcdc: u8;
            {
                lcdc = self.video_controller.borrow().lcdc;
            }
            if lcdc >> 7 == 0 {
                self.virtual_screen_layers.background=[[PaletteColor::Unset; 256]; 256];
            }
            let bg_tile_map: &[[u8; 32]; 32];
            if lcdc & 0x10 > 0 {
                //
                bg_tile_map = &self.vram.tmap2.tiles;
            } else {
                bg_tile_map = &self.vram.tmap1.tiles;
            }
            let pixel_map = |x: u16| Ppu::pre_bg_palette(x);
            let tile_map = |x: u8| {
                match x {
                    0..=0x7f => {
                        if lcdc & 0x10 > 0 {
                            self.vram.block0.objects[x as usize]
                        } else {
                            self.vram.block2.objects[x as usize]
                        }
                    }
                    0x80..=0xFF => self.vram.block1.objects[(x - 128) as usize],
                }
                .get_tile()
                .map(pixel_map)
            };
            let bg_tiles: [[[[PaletteColor; 8]; 8]; 32]; 32] = bg_tile_map.map(|x| x.map(|y| tile_map(y)));
            for x in 0..32 {
                for y in 0..32 {
                    for x_p in 0..8 {
                        for y_p in 0..8 {
                            self.virtual_screen_layers.background[y * 8 + y_p][x * 8 + x_p] = bg_tiles[y][x][y_p][x_p]
                        }
                    }
                }
            }
        }
        fn bg_palette(bgp: u8) -> HashMap<PaletteColor,GBColor>{

            let mut mapping = HashMap::<PaletteColor,GBColor>::new();
            let color_mapping = |x| match x{
                0 => GBColor::White,
                1 => GBColor::LightGrey,
                2 => GBColor::DarkGrey,
                3 => GBColor::Black,
                _=>unreachable!()
            };
            mapping.insert(PaletteColor::Zero, color_mapping( bgp & 0x03));
            mapping.insert(PaletteColor::One, color_mapping(bgp & 0x0C >> 2));
            mapping.insert(PaletteColor::Two, color_mapping(bgp & 0x30 >> 4));
            mapping.insert(PaletteColor::Three, color_mapping(bgp >> 6));
            return mapping;
        }
        fn obj_palette(bgp: u8) -> HashMap<PaletteColor,GBColor> {
            let mut mapping = HashMap::<PaletteColor,GBColor>::new();
            let color_mapping = |x| match x{
                0 => GBColor::Transparent,
                1 => GBColor::LightGrey,
                2 => GBColor::DarkGrey,
                3 => GBColor::Black,
                _=>unreachable!()
            };
            mapping.insert(PaletteColor::Zero, color_mapping( bgp & 0x03));
            mapping.insert(PaletteColor::One, color_mapping(bgp & 0x0C >> 2));
            mapping.insert(PaletteColor::Two, color_mapping(bgp & 0x30 >> 4));
            mapping.insert(PaletteColor::Three, color_mapping(bgp >> 6));
            return mapping
            
        }
        fn mode(&mut self) -> u8 {
            self.video_controller.borrow().stat % 4
        }

        //These functions have to be different because we have to implement PPU mode memory locking
        pub fn read_vram(&mut self, addr: u16) -> u8 {
            if self.mode() != 3 {
                return self.vram.memory_map(addr);
            }
            0xFF
        }
        pub fn write_vram(&mut self, addr: u16, val: u8) {
            if self.mode() != 3 {
                self.vram.memory_write(addr, val);
            }
        }
        pub fn read_oam(&mut self, addr: u16) -> u8 {
            if (0..=1).contains(&self.mode()) {
                return self.oam.memory_map(addr);
            }
            0xFF
        }
        pub fn write_oam(&mut self, addr: u16, val: u8) {
            if (0..=1).contains(&self.mode()) {
                self.vram.memory_write(addr, val);
            }
        }
    }
    struct Lcdc {
        lcd_enable: bool,
        window_tile_map: bool, //9800-0bff or 9c00-9fff
        window_enable: bool,
        bg_tile_data_area: bool, //8800-97ff
        obj_size: bool,
        obj_enable: bool,
        bg_window_enable: bool,
    }
    struct Stat {
        lyc_int_select: bool,
        mode_2_stat_int: bool,
        mode_1_stat_int: bool,
        mode_0_stat_int: bool,
        lylyc: bool,
        ppu_mode: u8,
    }
}
