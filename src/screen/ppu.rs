pub mod ppu {
    use std::{cell::RefCell, rc::Rc};

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
        pub bgp: u8,  //Background Pallete Data
        pub obp0: u8, //Oboject palette 1
        pub obp1: u8, //Object palette 2
        pub wy: u8,   //Window Y
        pub wx: u8,   //Window X
    }
    impl AsMemory for VideoController {
        //Remember
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0 => self.lcdc,
                1 => self.stat,
                2 => self.scy,
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
                1 => self.stat = val,
                2 => self.scy = val,
                3 => self.scx = val,  //Background viewport X
                4 => self.ly = val,   //Line of drawing
                5 => self.lyc = val,  //Line to compare
                6 => self.dma = val,  //DMA!
                7 => self.bgp = val,  //Background Pallete Data
                8 => self.obp0 = val, //Oboject palette 1
                9 => self.obp1 = val, //Object palette 2
                0xa => self.wy = val, //Window Y
                0xb => self.wx = val, //Window X
                _ => unreachable!(),
            }
        }
    }
    type VirtualInternalScreen = [[PixelColor; 256]; 256];
    type VirtualBoundedScreen = [[PixelColor; 160]; 144];
    pub struct Ppu {
        //This gets to be _fun_
        virtual_unrendered_screen: VirtualBoundedScreen,
        virtual_rendered_screen: VirtualBoundedScreen,
        pub vram: Vram,
        pub oam: OamStruct,
        dots: u16,
        mode_3_penalty: u16,
        video_controller: Rc<RefCell<VideoController>>, //vram:[u8;0x1800],
                                                        //oam:[[u8;4];40],
    }
    #[derive(Clone, Copy)]
    pub enum PixelColor {
        White,
        LightGrey,
        DarkGrey,
        Black,
        Transparent,
    }
    pub struct VirtualScreen {
        background: VirtualInternalScreen,
        window: VirtualInternalScreen,
    }
    impl Ppu {
        pub fn new(vc: Rc<RefCell<VideoController>>) -> Self {
            return Self {
                virtual_bounded_screen: [[PixelColor::Transparent; 160]; 144],
                vram: Vram::new(),
                oam: OamStruct::new(),
                dots: 0, //0..456
                video_controller: vc,
                mode_3_penalty: 0,
                virtual_internal_screen: VirtualScreen {
                    background: todo!(),
                    window: todo!(),
                },
            };
        }
        pub fn get_screen(&self) -> VirtualBoundedScreen {
            return self.virtual_bounded_screen;
        }
        pub fn hblank(&mut self) {}
        pub fn vblank(&mut self) {}
        pub fn oam_scan(&mut self) {
            //Figure out the OAM, save that data.
        }
        pub fn draw_pixels(&mut self) {
            let mut ret: VirtualBoundedScreen = [[PixelColor::Transparent; 160]; 144];
            let vc;
            {
                vc = self.video_controller.borrow().clone();
            }
            let window_enabled = vc.lcdc >> 5 % 2 == 1;
            if vc.lcdc % 2 == 1 {
                let background = self.generate_background();
                for y in 0..160 {
                    for x in 0..144 {
                        ret[y][x] =
                            background[(vc.scy as usize + y) % 256][vc.scx as usize + x % 256]
                    }
                }
                if window_enabled {
                    let window = self.generate_window();
                    for y in 0..160 - vc.wy {
                        for x in 0..144 - vc.wx {
                            if x - 7 + vc.wx > 0 {
                                ret[(vc.wy + y) as usize][(x - 7 + vc.wx) as usize] =
                                    window[y as usize][x as usize]
                            }
                        }
                    }
                }
                //Background is on, check background first.
            }

            let objects: [Option<Oam>; 10] = [None; 10];
            if (vc.lcdc >> 1) % 2 == 1 {
                //Obj is on
                if (vc.lcdc >> 2) % 2 == 1 {
                    //Obj is sized with 8x16
                } else {
                }
            }
            self.virtual_unrendered_screen = ret;
        }
        pub fn draw_dot(&mut self) -> Option<Interrupt> {
            let mut interrupt = None;
            let stat: u8;
            let lcdc: u8;
            let ly: u8;
            {
                let vc = self.video_controller.borrow();
                stat = vc.stat;
                lcdc = vc.lcdc;
                ly = vc.ly;
            }
            {
                let mut mode_3_transition: bool = false;
                {
                    let mut vc = self.video_controller.borrow_mut();
                    self.dots += 1; //Increase the dot
                    if self.dots == 80 && ly < 144 {
                        vc.stat += 1; //2->3
                        mode_3_transition = true;
                    }
                }
                if mode_3_transition {
                    self.draw_pixels();
                }
                {
                    let mut vc = self.video_controller.borrow_mut();

                    if self.dots > 456 {
                        //On row adjustment
                        self.dots = 0;
                        vc.ly += 1;
                        if vc.ly == vc.lyc {
                            vc.stat |= 0x04; //Set bit 2 to true
                            if vc.stat & 0x40 > 0 {
                                //Test if LCDC bit 6 is active
                                interrupt = Some(Interrupt::LCDC)
                            }
                        } else {
                            vc.stat &= 0xFB //Turn off bit 2
                        }
                        if vc.ly < 144 {
                            //We go from 0->2
                            vc.stat += 2;

                            if vc.stat & 0x20 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt = Some(Interrupt::LCDC)
                            }
                        }
                        if vc.ly == 144 {
                            //We're in 0->1
                            vc.stat += 1;
                            self.virtual_bounded_screen = [[PixelColor::Transparent; 160]; 144]; //Reset the internal screen
                            interrupt = Some(Interrupt::VBlank)
                        }
                        if vc.ly > 153 {
                            //1->2
                            vc.ly = 0;
                            vc.stat += 1;
                            if vc.stat & 0x20 > 0 {
                                //Test if LCDC bit 5 is active
                                interrupt = Some(Interrupt::LCDC)
                            }
                        }
                    }
                }
                return interrupt;
            }
        }
        fn generate_window(&mut self) -> VirtualInternalScreen {
            //let pre_textured :[[u8;256];256]=[[0;256];256];
            let lcdc: u8;
            {
                lcdc = self.video_controller.borrow().lcdc;
            }
            if lcdc >> 7 == 0 {
                return [[PixelColor::White; 256]; 256];
            }
            let bg_tile_map: &[[u8; 32]; 32];
            if lcdc & 0x40 > 0 {
                //
                bg_tile_map = &self.vram.tmap2.tiles;
            } else {
                bg_tile_map = &self.vram.tmap1.tiles;
            }

            let bgp: u8;
            {
                bgp = self.video_controller.borrow().bgp;
            }
            let pixel_map = |x: u16| Ppu::bg_palette(bgp, x);
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
            let mut ret: [[PixelColor; 256]; 256] = [[PixelColor::White; 256]; 256];

            for x in 0..32 {
                for y in 0..32 {
                    for x_p in 0..8 {
                        for y_p in 0..8 {
                            ret[y * 8 + y_p][x * 8 + x_p] = bg_tiles[y][x][y_p][x_p]
                        }
                    }
                }
            }
            return ret;
        }

        fn generate_background(&mut self) -> VirtualInternalScreen {
            //let pre_textured :[[u8;256];256]=[[0;256];256];
            let lcdc: u8;
            {
                lcdc = self.video_controller.borrow().lcdc;
            }
            if lcdc >> 7 == 0 {
                return [[PixelColor::White; 256]; 256];
            }
            let bg_tile_map: &[[u8; 32]; 32];
            if lcdc & 0x10 > 0 {
                //
                bg_tile_map = &self.vram.tmap2.tiles;
            } else {
                bg_tile_map = &self.vram.tmap1.tiles;
            }

            let bgp: u8;
            {
                bgp = self.video_controller.borrow().bgp;
            }
            let pixel_map = |x: u16| Ppu::bg_palette(bgp, x);
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
            let mut ret: [[PixelColor; 256]; 256] = [[PixelColor::White; 256]; 256];

            for x in 0..32 {
                for y in 0..32 {
                    for x_p in 0..8 {
                        for y_p in 0..8 {
                            ret[y * 8 + y_p][x * 8 + x_p] = bg_tiles[y][x][y_p][x_p]
                        }
                    }
                }
            }
            return ret;
        }
        //fn return_map(&self) -> [[PixelColor; 160]; 144] {
        //}
        fn bg_palette(bgp: u8, mut color: u16) -> [PixelColor; 8] {
            //
            let mut ret = [PixelColor::White; 8];
            for i in 0..8 {
                let pixel = color & 0x03;
                let background = match pixel {
                    0 => bgp & 0x03,
                    1 => bgp & 0x0C >> 2,
                    2 => bgp & 0x30 >> 4,
                    3 => bgp >> 6,
                    _ => unreachable!(),
                };

                ret[7 - i] = match background {
                    0 => PixelColor::White,
                    1 => PixelColor::LightGrey,
                    2 => PixelColor::DarkGrey,
                    3 => PixelColor::Black,
                    _ => unreachable!(),
                };
                color = color >> 2
            }
            ret
        }
        fn obj_palette(bgp: u8, mut color: u16) -> [PixelColor; 8] {
            //
            let mut ret = [PixelColor::White; 8];
            for i in 0..8 {
                let pixel = color & 0x03;
                let background = match pixel {
                    0 => bgp & 0x03,
                    1 => bgp & 0x0C >> 2,
                    2 => bgp & 0x30 >> 4,
                    3 => bgp >> 6,
                    _ => unreachable!(),
                };

                ret[7 - i] = match background {
                    0 => PixelColor::Transparent,
                    1 => PixelColor::LightGrey,
                    2 => PixelColor::DarkGrey,
                    3 => PixelColor::Black,
                    _ => unreachable!(),
                };
                color = color >> 2
            }
            ret
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
