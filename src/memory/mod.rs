pub mod mapped_io;
pub mod memory_wrapper {

    use std::{cell::RefCell, fs::File, io::Read, rc::Rc};

    use log::info;
    use sdl2::{render::Canvas, video::Window};

    use super::mapped_io::mapped_io::{self, MappedIO, OnClock};
    use crate::{
        audio::audio_controller::AudioController,
        cartridge::cartridge::Cartridge,
        joypad::joypad,
        screen::{
            screen::Screen,
            video_controller::video_controller::VideoController,
        },
    };
    //use crate::mapped_io;
    pub struct DmaTransfer {
        active: bool,
        addr_u: u16,
        addr_l: u16,
    }
    pub struct MemWrap {
        boot_rom: [u8; 0xff],
        cart: Cartridge,
        work_ram: [u8; 8192], // For CGB, switchable bank
        mapped_io: MappedIO,
        ppu: Screen,
        high_ram: [u8; 127],
        dma: DmaTransfer,
        wait: Rc<RefCell<u8>>,
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
                0x0000..=0x00ff => {
                    if self.mapped_io.boot_control == 0 {
                        self.boot_rom[addr as usize]
                    } else {
                        self.cart.memory_map(addr)
                    }
                }
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
                if (0x8000..=0x97FF).contains(&addr){
                    info!("WE WRITE TO BLOCKS  {:X?} @ {:X?} ", val, &addr)

                }
                match addr {
                    0x0000..=0x7FFF => self.cart.memory_write(addr, val), //ROM
                    0x8000..=0x9FFF => self.ppu.write_vram(addr - 0x8000, val), //VRAM
                    0xA000..=0xBFFF => self.cart.memory_write(addr - 0xa000, val), //External bus
                    0xC000..=0xDFFF => self.work_ram[(addr - 0xc000) as usize] = val, //WRAM
                    0xE000..=0xFDFF => todo!(),                           //ECHO
                    0xFE00..=0xFE9F => self.ppu.write_oam(addr - 0xFE00, val), // OAM
                    0xFEA0..=0xFEFF => unreachable!(),                           //Invalid OAM region
                    0xFF00..=0xFF7F | 0xFFFF => self.mapped_io.memory_write(addr - 0xFF00, val), //Memory mapped ram
                    0xFF80..0xFFFF => self.high_ram[(addr - 0xFF80) as usize] = val, //High ram
                }
            }
        }
    }
    impl MemWrap {
        pub fn new(
            joypad: Rc<RefCell<joypad::Joypad>>,
            audio: AudioController,
            canvas:Canvas<Window>,
            wait_ref: Rc<RefCell<u8>>,
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
                stat: 0x02,
            }));
            let mut boot_rom = [0 as u8; 0xff];
            let filename = "./src/memory/DMG_ROM.bin";
            let mut f = File::open(&filename).expect("no file found"); //Len = 256
            f.read(&mut boot_rom).expect("buffer overflow");
            Self {
                //rom: [0; 16384],
                cart: Cartridge::new(cartridge),
                boot_rom: boot_rom,
                //exrom: ExROM::new(cart_contents.clone()),
                mapped_io: mapped_io::MappedIO::new(joypad, audio, vcontroller.clone()),
                //external_ram: ExRam::new(cart_contents),
                work_ram: [0; 8192],
                high_ram: [0; 127],
                ppu: Screen::new(vcontroller,canvas),
                dma: DmaTransfer {
                    active: false,
                    addr_u: 0,
                    addr_l: 0,
                },
                wait: wait_ref,
            }
        }
        pub fn grab_memory_8(&mut self, addr: u16) -> u8 {
            //info!("GET_MEMORY_8 ADDR{:#x}", addr);
            *self.wait.borrow_mut() += 2;
            self.memory_map(addr)
            //self.my_memory[addr as usize]
        }
        pub fn grab_memory_16(&mut self, addr: u16) -> u16 { //Well, that's one mystery solved.
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
            *self.wait.borrow_mut() += 4;
            //info!("GET_MEMORY_16 ADDR{:#x}", addr);
            let high_byte = self.memory_map(addr+1);
            let low_byte = self.memory_map(addr);
            (high_byte as u16) * (1 << 8) + (low_byte as u16)
        }
        pub fn set_memory_8(&mut self, addr: u16, value: u8) {
            self.memory_write(addr, value);
        }
        pub fn set_memory_16(&mut self, addr: u16, value: u16) {
            self.memory_write(addr+1, (value >> 8) as u8);
            self.memory_write(addr, (value % (1 << 8)) as u8);
        }
       
        pub fn on_clock(&mut self) {
            self.ppu.on_clock();
            self.mapped_io.on_clock();
            if self.dma.active{
                self.dma()
            }
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
