//
//use crate::Surface
pub mod oam;
pub mod pixelqueue;
pub mod video_controller;
pub mod vram;
pub mod screen {


    use log::{debug, info};
    use sdl2::{pixels::Color, rect::Point, render::Canvas, video::Window};

    use crate::{
        cpu::cpu::Interrupt, memory::memory_wrapper::AsMemory, screen::{
            oam::oam::OamStruct, video_controller::video_controller::VideoController,
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
        pub fn get_screen(&self) -> VirtualBoundedScreen {
            //println!("We get the screen");
            return self.virtual_rendered_screen;
        }
        //fn hblank(&mut self) {}
        fn dirtiest_vblank(&mut self) {
            let mut background: [[Color; 256]; 256] = [[Color::RED; 256]; 256];
            let tile_data_area = match ((self.vc.lcdc >> 4) % 2) != 0 {
                true => (&self.vram.block0, &self.vram.block1),
                false => (&self.vram.block2, &self.vram.block1),
            };
            
            let background_palette = |x| match match x {
                ColorID::Zero => self.vc.bgp & 0x03,
                ColorID::One => (self.vc.bgp & 0x0C) >> 2,
                ColorID::Two => (self.vc.bgp & 0x30) >> 4,
                ColorID::Three => self.vc.bgp >> 6,
                ColorID::Unset => unreachable!(),
            } {
                0 => GBColor::White,
                1 => GBColor::LightGrey,
                2 => GBColor::DarkGrey,
                3 => GBColor::Black,
                _ => unreachable!(),
            };
            let tile_data_lookup = |x: u8| {
                Block::get_tile(match x {
                    0..128 => tile_data_area.0.objects[x as usize],
                    128..=255 => tile_data_area.1.objects[(x - 128) as usize],
                })
                .map(|x| x.map(|y| background_palette(y)))
            };

            let bg_tile_map = match self.vc.lcdc >> 3 % 2 == 0 {
                true => &self.vram.tmap2.tiles,
                false=> &self.vram.tmap1.tiles,
            };
            for tile_idx in 0..(32) {
            //let tile_idx = 0; 
                let row = self.vc.scy>>5;
                //dbg!("DRAW TILE {:X?}, Y={:X?}, X={:X?}",tile_idx, tile_idx %32, tile_idx >>5);
                let tile = tile_data_lookup(bg_tile_map[(32*row as usize %256)+tile_idx as usize]);
                for pixel_x in 0..8 {
                    let pixel_y:usize = self.vc.ly as usize% 8;
                   self.virtual_unrendered_screen[(tile_idx as usize >>5)*8+pixel_y ][(tile_idx%32)*8+pixel_x] =
                        tile[pixel_y][pixel_x];
                }
            }
            //}
            
            //info!("{:X?}",self.vc.scy);
            //thread::sleep(Duration::from_secs(5));/* 
           /*  for y in (self.vc.scy as u16)..=(self.vc.scy as u16 + 256){
                for x in (self.vc.scx as u16)..=(self.vc.scx as u16 + 256) {
                    self.virtual_unrendered_screen[self.vc.ly as usize] = background [self.vc.lyc as usize]
                }
            }   */
        }
        fn dirty_vblank(&mut self) {
            self.canvas.clear();
            let mut background: [[Color; 256]; 256] = [[Color::WHITE; 256]; 256];
            let tile_data_area = match ((self.vc.lcdc >> 4) % 2) != 0 {
                true => (&self.vram.block0, &self.vram.block1),
                false => (&self.vram.block2, &self.vram.block1),
            };
            
            let background_palette = |x| match match x {
                ColorID::Zero => self.vc.bgp & 0x03,
                ColorID::One => (self.vc.bgp & 0x0C) >> 2,
                ColorID::Two => (self.vc.bgp & 0x30) >> 4,
                ColorID::Three => self.vc.bgp >> 6,
                ColorID::Unset => unreachable!(),
            }{
                0 => Color::RGB(0xFF, 0xFF, 0xFF),
                1 => Color::RGB(0xb8, 0xb8, 0xb8),
                2 => Color::RGB(0x68, 0x68, 0x68),
                3 => Color::RGB(0x00, 0x00, 0x00),
                _ => unreachable!(),
            };
            let tile_data_lookup = |x: u8| {
                Block::get_tile(match x {
                    0..128 => tile_data_area.0.objects[x as usize],
                    128..=255 => tile_data_area.1.objects[(x - 128) as usize],
                })
                .map(|x| x.map(|y| background_palette(y)))
            };

            let bg_tile_map = match self.vc.lcdc >> 3 % 2 == 0 {
                true => &self.vram.tmap2.tiles,
                false=> &self.vram.tmap1.tiles,
            };
            for tile_idx in 0..(32*32) {
            //let tile_idx = 0; 
                //dbg!("DRAW TILE {:X?}, Y={:X?}, X={:X?}",tile_idx, tile_idx %32, tile_idx >>5);
                let tile = tile_data_lookup(bg_tile_map[tile_idx]);
                for pixel_x in 0..8 {
                    for pixel_y in 0..8 {
                        background[(tile_idx >>5)*8+pixel_y][(tile_idx%32)*8+pixel_x] =
                            tile[pixel_y][pixel_x];
                    }
                }
            }
            //}
            
            //info!("{:X?}",self.vc.scy);
            //thread::sleep(Duration::from_secs(5));
            for y in (self.vc.scy as u16)..=(self.vc.scy as u16 + 144){
                for x in (self.vc.scx as u16)..=(self.vc.scx as u16 + 160) {
                    self.canvas.set_draw_color(background[ (y%256) as usize][(x%256) as usize]); 
                    self.canvas
                        .draw_point(Point::new((((x-self.vc.scx as u16).wrapping_sub(7)as u16)%256) as i32, ((y-self.vc.scy as u16)%256) as i32))
                        .expect("Pixel failed to write");
                }
            } 
            self.canvas.present();
            self.virtual_unrendered_screen = [[GBColor::Transparent; 160]; 144];
        }
        pub fn vblank(&mut self) {
            self.canvas.clear();
            let screen_color = |x| match x {
                GBColor::White => Color::RGB(0xFF, 0xFF, 0xFF),
                GBColor::LightGrey => Color::RGB(0xb8, 0xb8, 0xb8),
                GBColor::DarkGrey => Color::RGB(0x68, 0x68, 0x68),
                GBColor::Black => Color::RGB(0x00, 0x00, 0x00),
                GBColor::Transparent => Color::RGB(0xff, 0x11, 0x11),
            };
            for y in 0..144 {
                for x in 0..160 {
                    self.canvas
                        .set_draw_color(screen_color(self.virtual_unrendered_screen[y][x]));
                    self.canvas
                        .draw_point(Point::new(x as i32, y as i32))
                        .expect("Pixel failed to write");
                }
            }
            self.canvas.present();
            self.virtual_unrendered_screen = [[GBColor::Transparent; 160]; 144];
            //Reset the internal screen
        }
        
        pub fn oam_scan(&mut self) {
            //info!("WE SCAN OAM");

            if self.objs.1 == 10 {
                return;
            }
            //info!("dots:{:?}", self.dots);
            if (self.vc.lcdc >> 1) % 2 == 1 && self.dots % 2 == 0 {
                let big = (self.vc.lcdc >> 2) % 2 == 1;
                let oam = self.oam.oam_list[(self.dots >> 1) as usize]; //We may need to truncate this?
                                                                        //info!("OAM is{:?}",oam);

                //thread::sleep(Duration::from_secs(5));
                let count: usize = self.objs.1 as usize;
                if (oam.ypos..oam.ypos + 8 + (8 * (big as u8))).contains(&self.vc.ly) {
                    self.objs.0[count] = Some((self.dots >> 1) as u8);
                    self.objs.1 += 1;
                }
            }

            //info!("OUR OAM IS {:?}", self.objs);
        }
        fn draw_line(&mut self) {
            //Run Mode 3 algorithm
            let bgp = self.vc.bgp;
            let color_mapping = |x| match x {
                0 => GBColor::White,
                1 => GBColor::LightGrey,
                2 => GBColor::DarkGrey,
                3 => GBColor::Black,
                _ => unreachable!(),
            };
            let background_palette = |x|match x {
                ColorID::Zero => color_mapping(bgp & 0x03),
                ColorID::One => color_mapping((bgp & 0x0C) >> 2),
                ColorID::Two => color_mapping((bgp & 0x30) >> 4),
                ColorID::Three => color_mapping(bgp >> 6),
                ColorID::Unset => unreachable!(),
            };

            let mut pixel_line: [GBColor; 160] = [GBColor::Transparent; 160];
            //let mut background: [GBColor; 256] = [GBColor::Transparent; 256]; //A leaner system, I think
            //let mut window: [GBColor; 256] = [GBColor::Transparent; 256];
            
            if self.vc.lcdc % 2 == 1 {
                //Actually work with window/background
                let tile_data_area = match ((self.vc.lcdc >> 4) % 2) != 0 {
                    true => (&self.vram.block0, &self.vram.block1),
                    false => (&self.vram.block2, &self.vram.block1),
                };
                let tile_data_lookup = |x: u8| {
                    Block::get_tile(match x {
                        0..128 => tile_data_area.0.objects[x as usize],
                        128..=255 => tile_data_area.1.objects[(x - 128) as usize],
                    })
                };
              
                
                let bg_tile_map = match self.vc.lcdc >> 3 % 2 == 0 {
                    true => &self.vram.tmap2.tiles,
                    false => &self.vram.tmap1.tiles,
                };
                //So If we are drawing pixel X, on line Y, this is:
                //PX = (BGX + SCX) % 256
                //LY = (BGY + SCY) % 256
                //LY-SCY = BGY % 256
                //BGX = BGX % 8 + BGX >>5 as the row index of the tile lookup
                //BGY = BGY % 8 + BGY >> 5 * 32 as the tile index
                //info!("WE GET HERE?");
                let bgy:usize =  self.vc.ly.wrapping_sub(self.vc.scy) as usize;
                for i in 0..(32*32){
                    let tile = tile_data_lookup(bg_tile_map[i]);
                    for j in tile{
                        for k in j{
                            if background_palette(k) != GBColor::White{
                                info!("COLOR FOUND IN TILE {}",i);
                            }
                        }
                    }
                }

                info!("LY = {:?},TILE_IDX {:?}",self.vc.ly,(bgy>>5));
                for px in 0..144{
                    let bgx:usize = (px as u8).wrapping_sub(self.vc.scx) as usize;
                    let tile_idx: usize = (32*(bgy>>5))+(bgx >>5);
                    let tile = tile_data_lookup(bg_tile_map[tile_idx]);
                    pixel_line[px as usize] = background_palette(tile[bgy as usize%8][bgx as usize%8]);
                    if pixel_line[px]!=GBColor::White{
                        info!("{:?}",pixel_line[px]);
                    }
                }

            }


/* 

                let row = self.vc.scy>>5;
                let pixel_y:usize = self.vc.ly as usize% 8;
                for tile_idx in 0..(32) {
                    //let tile_idx = 0; 
                        //dbg!("DRAW TILE {:X?}, Y={:X?}, X={:X?}",tile_idx, tile_idx %32, tile_idx >>5);
                        let tile = tile_data_lookup(bg_tile_map[(32*row as usize %256)+tile_idx as usize]);
                        for pixel_x in 0..8 {
                                background[(tile_idx as usize >>5)*8+pixel_y ][(tile_idx%32)*8+pixel_x] =
                                    tile[pixel_y][pixel_x];
                            
                        }
                }
                 
                if self.vc.lcdc >> 5 % 2 != 0 && self.vc.wy < self.vc.ly {
                    for i in 0..160 - self.vc.wx - 7 {
                        if (self.vc.wx as i8) - 7 > 0 {
                            info!("WX:{:#X?}, PIXEL ID:{:#X?}", self.vc.wx, i);
                            pixel_line[(self.vc.wx - 7 + i) as usize] = window[i as usize]
                        }
                    }
                }
 
                   if self.vc.lcdc >> 5 % 2 != 0 && self.vc.wy < self.vc.ly {
                    //Window
                    let window_tile_map = match self.vc.lcdc >> 5 % 2 == 0 {
                        true => &self.vram.tmap2,
                        false => &self.vram.tmap1,
                    };

                    let tile_y = self.vc.wy >> 3;
                    let in_tile_row = self.vc.wy % 8;
                    for tile_idx in 32*tile_y..32*(tile_y+1){
                        for in_tile_x in 0..8 {
                            window[(8 * tile_idx + in_tile_x) as usize] = background_palette(
                                tile_data_lookup(window_tile_map.tiles[tile_idx as usize])
                                    [in_tile_row as usize][in_tile_x as usize],
                            );
                        }
                    } 
                } 
            }*//* 
            if self.vc.lcdc >> 1 % 2 != 0 { //Do we draw objects.
                let obj_size = self.vc.lcdc >> 2 % 2 == 1;
                let vobj_lookup = |x: u8| match x {
                    0..128 => self.vram.block0.objects[x as usize],
                    128..=255 => self.vram.block1.objects[(x - 128) as usize],
                };
                let object_palette = |x, obp: u8| match x {
                    ColorID::Zero => GBColor::Transparent,
                    ColorID::One => color_mapping(obp & 0x0C >> 2),
                    ColorID::Two => color_mapping(obp & 0x30 >> 4),
                    ColorID::Three => color_mapping(obp >> 6),
                    ColorID::Unset => unreachable!(),
                };
                for obj_idx in (0..self.objs.1).rev() {
                    //this accounts for the full overlapping case, but not the X coordinate case.
                    let obj = self.objs.0[obj_idx as usize].expect("Failed to grab an object");
                    let oam_obj = self.oam.oam_list[obj as usize];
                    let vobj_1 = vobj_lookup(obj);
                    let vobj_2 = vobj_lookup(obj + 1); //We only use this if we're BIG
                    oam_obj.xpos;

                    let priority = oam_obj.attributes & 0x80 != 0;
                    let y_flip = oam_obj.attributes & 0x40 != 0;
                    let x_flip = oam_obj.attributes & 0x20 != 0;
                    let palette = |x| {
                        object_palette(
                            x,
                            match oam_obj.attributes & 0x10 != 0 {
                                true => self.vc.obp1,
                                false => self.vc.obp0,
                            },
                        )
                    };
                    let mut tile_1;
                    let mut tile_2;
                    if x_flip {
                        tile_1 = Block::get_tile_backwards(vobj_1);
                        tile_2 = Block::get_tile_backwards(vobj_2);
                    } else {
                        tile_1 = Block::get_tile(vobj_1);
                        tile_2 = Block::get_tile(vobj_2);
                    }
                    if y_flip {
                        if obj_size {
                            let mut storage = tile_1.clone();
                            storage.reverse();
                            tile_2.reverse();
                            tile_1 = tile_2;
                            tile_2 = storage;
                        } else {
                            tile_1.reverse();
                        }
                    }
                    let row_index = self.vc.ly - (oam_obj.ypos - (8)) % (4 + (4 * obj_size as u8)); //Double check everything, because one guide is saying sprites start at bottom right
                    let color_row = match row_index > 3 {
                        false => tile_1[row_index as usize],
                        true => tile_2[(row_index % 4) as usize],
                    };
                    let pixel_row = color_row.map(|x| (palette(x), priority));
                    for x_idx in 0..8 {
                        let left_corner = (oam_obj.xpos - 8) + x_idx;
                        if (0..168).contains(&left_corner) {
                            let (pixel, priority) = pixel_row[x_idx as usize];
                            let bg_0 = background_palette(ColorID::Zero);
                            let current_pixel = pixel_line[(left_corner + x_idx) as usize];
                            if priority || (current_pixel == bg_0) {
                                pixel_line[(left_corner + x_idx) as usize] = pixel;
                            }
                        }
                    }
                }
            } */

            self.virtual_unrendered_screen[self.vc.ly as usize] = pixel_line;
        }
        pub fn on_clock(&mut self) -> (Option<Interrupt>, Option<Interrupt>) {
            let mut ret = (None,None);
            if self.vc.lcdc >> 7 == 1 {
                for _i in 0..4{
                    let current_interation = self.ppu_dot_cycle();
                    ret.0 = match current_interation.0{
                        Some(x) => Some(x),
                        None => ret.0
                    };
                    ret.1 = match current_interation.1{
                        Some(x) => Some(x),
                        None => ret.1
                    };
                }
                
            }
            ret
        }
        pub fn ppu_dot_cycle(&mut self) -> (Option<Interrupt>, Option<Interrupt>) {
            let mut interrupt: (Option<Interrupt>, Option<Interrupt>) = (None, None);

            //info!("VC:{:?}", vc);
            //info!("DOTS:{:?}",self.dots);

            if self.vc.ly < 144 {
                if self.dots == 80 {
                    self.vc.stat += 1; //2->3
                    //self._vblank();
                    //self.draw_line()
                }
                if self.dots == 300 {
                    //3->0
                    self.vc.stat -= 3;
                    if self.vc.stat & 0x10 > 0 {
                        //Test if LCDC bit 5 is active
                        interrupt.0 = Some(Interrupt::LCDC);
                    }
                }
            }
            if self.mode() == 2 {
                self.oam_scan(); //This is when we start scanning the OAM
            }
            if self.dots > 455 {
                //On row adjustment
                self.dots = 0;
                self.vc.ly += 1;
                if self.vc.ly == self.vc.lyc {
                    //Manage LCY
                    self.vc.stat |= 0x04; //Set bit 2 to true
                    if self.vc.stat & 0x40 > 0 {
                        //Test if LCDC bit 6 is active
                        interrupt.0 = Some(Interrupt::LCDC);
                    }
                } else {
                    self.vc.stat &= 0xFB; //Turn off bit 2, turning off the LCDC
                }
                if self.vc.ly < 144 {
                    //mode 0->2
                    self.vc.stat += 2;
                    if self.vc.stat & 0x20 > 0 {
                        self.objs = ([None; 10], 0);
                        //Test if LCDC bit 5 is active
                        interrupt.0 = Some(Interrupt::LCDC);
                    }
                }
                if self.vc.ly == 144 {
                    //mode 0->1
                    self.vc.stat += 1;
                    self.dirty_vblank();
                    interrupt.1 = Some(Interrupt::VBlank); 
                }

                if self.vc.ly > 153 {
                    //mode 1->2
                    self.vc.ly = 0;
                    self.vc.stat += 1;
                    if self.vc.stat & 0x20 > 0 {
                        //Test if LCDC bit 5 is active
                        interrupt.0 = Some(Interrupt::LCDC);
                    }
                }
            }
            self.dots += 1; //Increase the dot
            return interrupt;
        }

        fn mode(&mut self) -> u8 {
            self.vc.stat % 4
        }
        //These functions have to be different because we have to implement PPU mode memory locking
        pub fn read_vram(&mut self, addr: usize) -> u8 {
            if self.mode() != 3 {
                return self.vram.memory_map(addr);
            }
            0xFF
        }
        pub fn write_vram(&mut self, addr: usize, val: u8) {
            if self.mode() != 3 {
                self.vram.memory_write(addr, val);
            } else {
                info!("Our issue is here!");
            }
        }
        pub fn read_oam(&mut self, addr: usize) -> u8 {
            if (0..=1).contains(&self.mode()) {
                return self.oam.memory_map(addr);
            }
            0xFF
        }
        pub fn write_oam(&mut self, addr: usize, val: u8) {
            if (0..=1).contains(&self.mode()) {
                self.vram.memory_write(addr, val);
            }
        }
    }
    /* struct Lcdc {
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
    } */
}
