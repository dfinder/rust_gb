pub mod oam {

    use crate::memory::memory_wrapper::AsMemory;
    pub struct OamStruct {
        pub oam_list: [Oam; 40],
    }
    impl OamStruct {
        pub fn new() -> Self {
            return Self {
                oam_list: [Oam {
                    ypos: 0,
                    xpos: 0,
                    tile_index: 0,
                    attributes: 0,
                }; 40],
            };
        }
    }

    #[derive(Clone, Copy)]
    pub struct Oam {
        pub ypos: u8,
        pub xpos: u8,
        pub tile_index: u8,
        pub attributes: u8,
    }
    impl AsMemory for Oam {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.ypos,
                0x0001 => self.xpos,
                0x0002 => self.tile_index,
                0x0003 => self.attributes,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000 => self.ypos = val,
                0x0001 => self.xpos = val,
                0x0002 => self.tile_index = val,
                0x0003 => self.attributes = val,
                _ => unreachable!(),
            }
        }
    }
    impl AsMemory for OamStruct {
        //Somehow we need to check if we're in VBlank or HBlank from FF41
        fn memory_map(&mut self, addr: u16) -> u8 {
            self.oam_list[(addr >> 2) as usize].memory_map(addr & 0x0003)
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            self.oam_list[(addr >> 2) as usize].memory_write(addr & 0x0003, val)
        }
    }
}
