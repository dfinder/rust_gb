pub mod video_controller {
    use crate::memory::memory_wrapper::AsMemory;
    use std::fmt::Debug;
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
    impl Debug for VideoController {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("VideoController")
                .field("lcdc", &format!("{:X?}", &self.lcdc))
                .field("stat", &format!("{:X?}", &self.stat))
                .field("scy", &format!("{:X?}", &self.scy))
                .field("scx", &format!("{:X?}", &self.scx))
                .field("ly", &format!("{:X?}", &self.ly))
                .field("lyc", &format!("{:X?}", &self.lyc))
                .field("dma", &format!("{:X?}", &self.dma))
                .field("bgp", &format!("{:X?}", &self.bgp))
                .field("obp0", &format!("{:X?}", &self.obp0))
                .field("obp1", &format!("{:X?}", &self.obp1))
                .field("wy", &format!("{:X?}", &self.wy))
                .field("wx", &format!("{:X?}", &self.wx))
                .finish()
        }
    }
    impl AsMemory for VideoController {
        //Remember
        fn memory_map(&mut self, addr: usize) -> u8 {
            match addr {
                0 => self.lcdc, //LCD control register, controls which banks are used, whether windows/background is used, etc.
                1 => self.stat, //Interrupts
                2 => self.scy,  //Background viewport Y
                3 => self.scx,  //Background viewport X
                4 => self.ly,   //Line of drawinginfo!("LY IS {:X?}"
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

        fn memory_write(&mut self, addr: usize, val: u8) {
            match addr {
                0 => self.lcdc = val,
                1 => self.stat = val & 0xFB, //Last two bits are unwritable
                2 => {
                    self.scy = val;
                    //info!("WE SET SCY{:?}", val)
                } //Background Viewport Y
                3 => self.scx = val,         //Background viewport X
                4 => (),                     //Line of drawing, is READ ONLY
                5 => self.lyc = val,         //Line to compare
                6 => self.dma = val,         //DMA! Controls a fun bypass between RAM and VRAM
                7 => self.bgp = val,         //Background Pallette Data
                8 => self.obp0 = val,        //Object palette 1
                9 => self.obp1 = val,        //Object palette 2
                0xa => self.wy = val,        //Window Y
                0xb => self.wx = val,        //Window X
                _ => unreachable!(),
            }
        }
    }
}
