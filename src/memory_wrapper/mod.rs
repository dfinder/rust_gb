pub mod mapped_io;
pub mod rom;
pub mod memory_wrapper {

    use std::{cell::RefCell, fs::File, rc::Rc};

    use super::mapped_io::mapped_io::{self, MappedIO};
    use crate::{
        audio::audio_controller::AudioController,
        cartridge::cartridge::Cartridge,
        joypad::joypad,
        screen::{ppu::ppu::{GBColor, Ppu,}, video_controller::video_controller::VideoController},
    };
    //use crate::mapped_io;
    pub struct DmaTransfer {
        active: bool,
        addr_u: u16,
        addr_l: u16,
    }
    pub struct MemWrap {
        cart: Cartridge,
        work_ram: [u8; 8192], // For CGB, switchable bank
        mapped_io: MappedIO,
        ppu: Ppu,
        high_ram: [u8; 127],
        dma: DmaTransfer,
        wait: Rc<RefCell<u8>>
    }
    pub trait AsMemory {
        fn memory_map(&mut self, addr: u16) -> u8;
        fn memory_write(&mut self, addr: u16, val: u8);
    }
    impl AsMemory for MemWrap {
        fn memory_map(&mut self, addr: u16) -> u8 {
            if self.dma.active {
                match addr {
                    //We can only access high ram during DMA transfer
                    0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize],
                    _ => 0xFF,
                };
            }
            match addr {
                0x0000..=0x7FFF => self.cart.memory_map(addr), //ROM
                0x8000..=0x9FFF => self.ppu.read_vram(addr - 0x8000), //VRAM
                0xA000..=0xBFFF => self.cart.memory_map(addr), //External bus
                0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize], //WRAM
                0xE000..=0xFDFF => todo!(),                    //ECHO
                0xFE00..=0xFE9F => self.ppu.read_oam(addr - 0xFE00), // OAM
                0xFEA0..=0xFEFF => todo!(),                    //Invalid OAM region
                0xFF00..=0xFF7F => self.mapped_io.memory_map(addr - 0xFF00), //Memory mapped ram
                0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize], //High ram
                0xFFFF => self.mapped_io.memory_map(0xFF),     //Interrupts
            }
        }
        fn memory_write(&mut self, addr: u16, val: u8) {
            if self.dma.active {
                match addr {
                    //We can only access high ram during DMA transfer
                    0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize] = val,
                    _ => (),
                };
            } else {
                if addr == 0xFF46 {
                    self.dma = DmaTransfer {
                        active: false,
                        addr_u: val as u16,
                        addr_l: 0,
                    }; //We, of course, cannot start a DMA transfer while one is ongoing
                }
                match addr {
                    0x0000..=0x7FFF => self.cart.memory_write(addr, val), //ROM
                    0x8000..=0x9FFF => self.ppu.write_vram(addr - 0x8000, val), //VRAM
                    0xA000..=0xBFFF => self.cart.memory_write(addr - 0xa000, val), //External bus
                    0xC000..=0xDFFF => self.work_ram[(addr - 0xc000) as usize] = val, //WRAM
                    0xE000..=0xFDFF => todo!(),                           //ECHO
                    0xFE00..=0xFE9F => self.ppu.write_oam(addr - 0xFE00, val), // OAM
                    0xFEA0..=0xFEFF => todo!(),                           //Invalid OAM region
                    0xFF00..=0xFF7F | 0xFFFF => self.mapped_io.memory_write(addr - 0xFF00, val), //Memory mapped ram
                    0xFF80..0xFFFF => self.high_ram[(addr - 0xFF80) as usize] = val, //High ram
                }
            }
        }
    }
    impl MemWrap {
        pub fn new(
            joypad: std::rc::Rc<RefCell<joypad::Joypad>>,
            audio: Rc<RefCell<AudioController>>,
            wait_ref:Rc<RefCell<u8>>,
            cartridge: File,
        ) -> Self {
            let vcontroller = Rc::new(RefCell::new(VideoController {
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
                stat: 0,
            }));
            Self {
                //rom: [0; 16384],
                cart: Cartridge::new(cartridge),
                //exrom: ExROM::new(cart_contents.clone()),
                mapped_io: mapped_io::MappedIO::new(joypad, audio, vcontroller.clone()),
                //external_ram: ExRam::new(cart_contents),
                work_ram: [0; 8192],
                high_ram: [0; 127],
                ppu: Ppu::new(vcontroller),
                dma: DmaTransfer {
                    active: false,
                    addr_u: 0,
                    addr_l: 0,
                },
                wait:wait_ref
            }
        }
        pub fn grab_memory_8(&mut self, addr: u16) -> u8 {
            *self.wait.borrow_mut()+=2;
            self.memory_map(addr)
            //self.my_memory[addr as usize]
        }
        pub fn grab_memory_16(&mut self, addr: u16) -> u16 {
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
            
            *self.wait.borrow_mut()+=4; //
            (self.memory_map(addr + 1) as u16) << 8 + (self.memory_map(addr + 1) as u16)
        }
        pub fn set_memory_8(&mut self, addr: u16, value: u8) {
            self.memory_write(addr, value);
        }
        pub fn set_memory_16(&mut self, addr: u16, value: u16) {
            self.memory_write(addr, (value >> 8) as u8);
            self.memory_write(addr, (value % (1 << 8)) as u8);
        }
        pub fn get_screen(&mut self) -> [[GBColor; 160]; 144] {
            self.ppu.get_screen()
        }
        pub fn on_clock(&mut self){
            self.ppu.ppu_dot_cycle();
        }
        pub fn dma(&mut self) {
            let addr: u16 = 0x0000 + ((self.dma.addr_u) << 8) + self.dma.addr_l;
            let source = match addr {
                0x0000..=0x7FFF => self.cart.memory_map(addr), //ROM
                0x8000..=0x9FFF => self.ppu.vram.memory_map(addr - 0x8000), //VRAM
                0xA000..=0xBFFF => self.cart.memory_map(addr), //External bus
                0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize],
                _ => unreachable!(),
            };
            self.ppu.oam.memory_write(self.dma.addr_l, source);
            self.dma.addr_l += 1;
            if self.dma.addr_l == 160 {
                self.dma = DmaTransfer {
                    active: false,
                    addr_u: 0,
                    addr_l: 0,
                }
            }
        }
    }
}
