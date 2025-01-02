pub mod ppu {
    use std::{cell::RefCell, rc::Rc};


    use crate::{
        interrupt::interrupt::Interrupt,
        memory_wrapper::memory_wrapper::AsMemory,
        screen::{
            oam::oam::{Oam, OamStruct},video_controller::video_controller::VideoController, vram::vram::Vram
        },
    };
   
    //type VirtualInternalScreen = [[Tile; 32]; 32];
    //type ObjScreen = [[ColorID; 160]; 144];
    type VirtualBoundedScreen = [[GBColor; 160]; 144];
  
    pub struct Ppu {
        virtual_unrendered_screen: VirtualBoundedScreen,
        virtual_rendered_screen: VirtualBoundedScreen,
        pub vram: Vram,
        pub oam: OamStruct,
        dots: u16,
        objs:([Option<u8>;10],u8),
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
    pub enum ColorID {
        Unset,
        Zero,
        One,
        Two,
        Three
    }

    /**pub struct PixelFetcher{
        mode_3_penalty:u8,
        background_fifo:PixelFIFO,
        obj_fifo:PixelFIFO,
        objs:([Option<u8>;10],u8),
        current_pixel:u8,
        stage:Stage,

    }**/
    impl Ppu {
        pub fn new(vc: Rc<RefCell<VideoController>>) -> Self {
            return Self {
                virtual_unrendered_screen: [[GBColor::Transparent; 160]; 144],
                virtual_rendered_screen: [[GBColor::Transparent; 160]; 144],
                vram: Vram::new(),
                oam: OamStruct::new(),
                dots: 0, //0..456
                video_controller: vc,
                objs: ([None;10],0),
            }
               
        }
        pub fn get_screen(&self) -> VirtualBoundedScreen {
            return self.virtual_rendered_screen;
        }
        pub fn hblank(&mut self) {}
        pub fn vblank(&mut self) {}
        pub fn oam_scan(&mut self){
            let vc;
            {
                vc = self.video_controller.borrow().clone();
            }
            if self.objs.1==10{
                return
            }

            if (vc.lcdc >> 1) % 2 == 1 && self.dots%2 ==0{
                let big = (vc.lcdc >> 2) % 2 == 1;
                let oam = self.oam.oam_list[(self.dots>>1) as usize];   
                let count:usize = self.objs.1 as usize;
                if (oam.ypos..oam.ypos+8+(8*(big as u8))).contains(&vc.ly){
                    self.objs.0[count]=Some((self.dots>>1) as u8);
                    self.objs.1+=1;
                }
            }
        }
        fn draw_line(&mut self){ //Run Mode 3 algorithm
            let bgp = self.video_controller.borrow().bgp;
            let color_mapping = |x| match x{
                0 => GBColor::White,
                1 => GBColor::LightGrey,
                2 => GBColor::DarkGrey,
                3 => GBColor::Black,
                _=>unreachable!()
            };
            let background_palette= |x| match x{
                ColorID::Zero=>color_mapping(bgp & 0x03),
                ColorID::One=> color_mapping(bgp & 0x0C >> 2),
                ColorID::Two=> color_mapping(bgp & 0x30 >> 4),
                ColorID::Three=> color_mapping(bgp >> 6),
                ColorID::Unset => unreachable!(),
            };
          
            let mut pixel_line = [GBColor::Transparent;160];
            let mut background:[GBColor;256] = [GBColor::Transparent;256]; //A leaner system, I think

            let mut window:[GBColor;256] = [GBColor::Transparent;256];
            let mut obj_layer: [[ColorID;160];144] = [[ColorID::Unset;160];144];
            
            //let mut canvas : [[GBColor;160];144]= [[GBColor::Transparent;160];144];

            let vc = self.video_controller.borrow().clone();

            if vc.lcdc %2 == 1{
                //Actually work with window/background
                let tile_data_area = match vc.lcdc >>4 %2 != 0{
                    true => (&self.vram.block0,&self.vram.block1),
                    false => (&self.vram.block2,&self.vram.block1)
                };
                let title_data_lookup = |x:u8| match x{
                    0..128=> tile_data_area.0.objects[x as usize],
                    128..=255=>tile_data_area.1.objects[(x-128) as usize],
                }.get_tile();
                if vc.lcdc>>5 %2 != 0 && vc.wy<vc.ly{ //Actually ues the window
                    let window_tile_map = match vc.lcdc>>5 %2 != 0 {
                        true => &self.vram.tmap2,
                        false => &self.vram.tmap1
                    };
                    for tile_x in 0..32{
                        let tile_y = vc.ly>>3;
                        let in_tile_row =vc.ly%8;
                        for in_tile_x in 0..8{
                            window[8*tile_x+in_tile_x]=
                                background_palette(title_data_lookup(window_tile_map.tiles[tile_y as usize][tile_x])[in_tile_row as usize][in_tile_x]);
                        }
                    }
                }
                let bg_tile_map = match vc.lcdc >> 3 %2 !=0{
                    true => &self.vram.tmap2,
                    false => &self.vram.tmap1
                };
                for tile_x in 0..32{
                    let view_port_y = vc.scy-vc.ly;
                    let tile_y = view_port_y>3;
                    let in_tile_row = view_port_y%8;
                    for in_tile_x in 0..8{
                        background[8*tile_x+in_tile_x]=
                            background_palette(title_data_lookup(bg_tile_map.tiles[tile_y as usize][tile_x])[in_tile_row as usize][in_tile_x]);
                    }
                }

                for i in 0..160{
                    pixel_line[i]=background[(i+vc.scx as usize) % 256]
                }
                if vc.lcdc>>5 %2 != 0 && vc.wy<vc.ly{
                    for i in 0..160-vc.wx-7{
                        pixel_line[(vc.wx-7+i) as usize] =window[i as usize]
                    }
                }
            }
            if vc.lcdc>>1 %2 !=0{

            
            let obp = |x:Oam| match x.attributes >>4 %2{
                1=> self.video_controller.borrow().obp1,
                0=> self.video_controller.borrow().obp0,
                _=>unreachable!(),
            };
            let object_palette= |x,obp:u8| match x{
                ColorID::Zero=>GBColor::Transparent,
                ColorID::One=> color_mapping(obp & 0x0C >> 2),
                ColorID::Two=> color_mapping(obp & 0x30 >> 4),
                ColorID::Three=> color_mapping(obp >> 6),
                ColorID::Unset => unreachable!(),
            };
            for obj_idx in 0..self.objs.1{
                let obj = self.objs.0[obj_idx as usize].expect("msg");
                let oam_obj = self.oam.oam_list[obj as usize];
                oam_obj.xpos;
                oam_obj.ypos;
                oam_obj.tile_index;
                oam_obj.attributes;
            }
        }
            for i in 0..=160{

            }
            self.virtual_unrendered_screen[self.video_controller.borrow().ly as usize] = pixel_line;

        }
        fn draw_pixel(&mut self,pixel:GBColor){
            self.virtual_unrendered_screen[self.video_controller.borrow().ly as usize][(self.dots-80) as usize]=pixel;
        }
        
        pub fn ppu_dot_cycle(&mut self) -> (Option<Interrupt>,Option<Interrupt>) {
            let mut interrupt:(Option<Interrupt>,Option<Interrupt>) = (None,None);
            let mut init_mode_2: bool = false;
            let mut init_mode_3: bool = false;
            let ly: u8;
            {
                let mut vc = self.video_controller.borrow_mut();
                ly = vc.ly;
                if self.dots == 0 && ly < 144 {
                    init_mode_2=true;
                }
                if self.dots == 80 && ly < 144 {
                    vc.stat += 1; //2->3    
                    init_mode_3=true
                }
            }
            {   
                if init_mode_2{
                    self.objs=([None;10],0);
                }
                if self.mode()==2 {
                    self.oam_scan(); //This is when we start scanning the OAM
                }
                if init_mode_3 {
                    self.draw_line(); //This is when we actually do the mode 3 work.
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
                        if vc.ly < 144 {//ENTER MODE 2
                            //mode 0->2
                            vc.stat += 2;
                            if vc.stat & 0x20 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt.0 = Some(Interrupt::LCDC)
                            }
                        }
                        if vc.ly == 144 {
                            //mode 0->1
                            vc.stat += 1;
                            self.virtual_rendered_screen=self.virtual_unrendered_screen;
                            self.virtual_unrendered_screen= [[GBColor::Transparent; 160]; 144]; //Reset the internal screen
                            interrupt.1 = Some(Interrupt::VBlank) //Remember, VBlank is a separate  
                        }
                        if vc.ly > 153 {
                            //mode 1->2
                            vc.ly = 0;
                            vc.stat += 1;
                            if vc.stat & 0x20 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt.0 = Some(Interrupt::LCDC)
                            }
                        }

                        if vc.ly > 220  {
                            //mode 3->0
                            vc.stat -=3;
                            if vc.stat & 0x10 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt.0 = Some(Interrupt::LCDC)
                            }
                        }
                    }
                }
                self.dots += 1; //Increase the dot
                return interrupt;
            }
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
