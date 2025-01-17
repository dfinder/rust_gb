pub mod memory_wrapper {

    use std::{cell::RefCell, fs::File, io::Read, rc::Rc};

    use crate::{
        audio::audio_controller::AudioController, cartridge::cartridge::Cartridge, cpu::interrupt::interrupt::Interrupt, joypad::joypad::Joypad, screen::screen::Screen
    };
    use sdl2::{render::Canvas, video::Window};

    struct Timer {
        divider: u16, //Divider The div is the visible part of the system counter
        tima: u8,         //Timer counter.
        tma: u8,          //Timer reload.
        tac: u8,          //Timer control
    }
    impl Timer{
        fn on_clock(&mut self)->Option<Interrupt> {
            self.divider = self.divider.wrapping_add(1);
            let frequency = match self.tac % 4 {
                0 => 8, //Every 256 m cycles
                1 => 2, //4 M cycles
                2 => 4, //16 m cycles
                3 => 6, //64 m cycles.
                _ => unreachable!(),
            };
            //If timer is enabled. If we hit 0 on the internal divider.
            if (self.tac & 0x04 > 0)
                && ((self.divider % (1 << frequency)) == 0)
            {
                let overflow: bool;
                (self.tima, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    self.tima = self.tma;
                    return Some(Interrupt::Timer);
                }
            }
            None
        }
    }
    impl AsMemory for Timer {
        fn memory_map(&mut self, addr: usize) -> u8 {
            match addr {
                0 => ((self.divider & 0xFF00) >> 8) as u8,
                1 => self.tima,
                2 => self.tma,
                3 => self.tac,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: usize, val: u8) {
            match addr {
                0 => self.divider = 0,
                1 => self.tima = val,
                2 => self.tma = val,
                3 => self.tac = val,
                _ => unreachable!(),
            }
        }
       
    }
    //use crate::mapped_io;
    pub struct DmaTransfer {
        active: bool,
        addr_u: usize,
        addr_l: usize,
    }
    pub struct MemWrap {
        boot_rom: [u8; 0x100],
        cart: Cartridge,
        work_ram: [u8; 8192], // For CGB, switchable bank
        pub ppu: Screen,
        high_ram: [u8; 127],
        dma: DmaTransfer,
        wait: Rc<RefCell<u8>>,
        joypad: Joypad, //FF00
        //serial: Serial,    //FF01, FF02 [FF03 is unmapped]
        timer: Timer,
        iflag: u8,
        audio_controller: AudioController,
        pub boot_control: u8,
        interrupts_enabled: u8, //LCDControl,
    }
    pub trait AsMemory {
        fn memory_map(&mut self, addr: usize) -> u8;
        fn memory_write(&mut self, addr: usize, val: u8);
    }
    impl AsMemory for MemWrap {
        fn memory_map(&mut self, addr: usize) -> u8 {
            if self.dma.active {
                match addr {
                    //We can only access high ram during DMA transfer
                    0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80)],
                    _ => 0xFF,
                };
            }
            match addr {
                0x0000..=0x00ff => {
                    if self.boot_control == 0 {
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
                0xFF00..=0xFF7F => match addr - 0xFF00 {
                    0x00 => self.joypad.read(),
                    0x01 => 0, //Serial
                    0x02 => 0,
                    0x03 => 0,
                    0x04..=0x07 => self.timer.memory_map(addr - 0xff04),
                    0x0f => 0xE0 | self.iflag,
                    0x10..0x26 => self.audio_controller.memory_map(addr - 0xff10),
                    0x40..=0x4b => self.ppu.vc.memory_map(addr - 0xff40),
                    //=>
                    0x50 => self.boot_control % 2,
                    0xff => self.interrupts_enabled,
                    _ => unreachable!(),
                }, //Memory mapped ram
                0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize], //High ram
                0xFFFF => self.iflag,     
                0x10000.. =>unreachable!()                     //Interrupts
            }
        }
        fn memory_write(&mut self, addr: usize, val: u8) {
            if self.dma.active {
                match addr {
                    //We can only access high ram during DMA transfer
                    0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize] = val,
                    _ => (),
                };
            } else {
                if addr == 0xFF46 {
                    self.dma = DmaTransfer {
                        active: true,
                        addr_u: val as usize,
                        addr_l: 0,
                    }; //We, of course, cannot start a DMA transfer while one is ongoing
                }
                match addr {
                    0x0000..=0x7FFF => self.cart.memory_write(addr, val), //ROM
                    0x8000..=0x9FFF => self.ppu.write_vram(addr - 0x8000, val), //VRAM
                    0xA000..=0xBFFF => self.cart.memory_write(addr - 0xa000, val), //External Ram
                    0xC000..=0xDFFF => self.work_ram[(addr - 0xc000) as usize] = val, //WRAM
                    0xE000..=0xFDFF => todo!(),                           //ECHO
                    0xFE00..=0xFE9F => self.ppu.write_oam(addr - 0xFE00, val), // OAM
                    0xFEA0..=0xFEFF => unreachable!(),                    //Invalid OAM region
                    0xFF00..=0xFF7F => match addr - 0xFF00 {
                        0x0000 => self.joypad.write(val),
                        0x0001 | 0x0002 => todo!(), //Serial
                        0x0003 => todo!(),          //Unmapped
                        0x0004..=0x0007 => self.timer.memory_write(addr - 0xff04, val),
                        0x000f => self.iflag = 0xE0 | val,
                        0x0010..=0x003f => self.audio_controller.memory_write(addr - 0xff10, val),

                        0x40..=0x4b => self.ppu.vc.memory_write(addr - 0xff40, val),
                        0x50 => self.boot_control = 0x01,
                        0xff => self.interrupts_enabled = val,
                        _ => unreachable!(),
                    }, //Memory mapped ram
                    0xFF80..0xFFFF => self.high_ram[(addr - 0xFF80) as usize] = val, //High ram
                    0xFFFF => self.iflag = val,

                    0x10000.. =>unreachable!()   
                }
            }
        }
    }
    impl MemWrap {
        pub fn new(
            joypad: Joypad,
            audio: AudioController,
            canvas: Canvas<Window>,
            wait_ref: Rc<RefCell<u8>>,
            cartridge: File,
        ) -> Self {
            let mut boot_rom = [0 as u8; 0x100];
            let filename = "./src/memory/DMG_ROM.bin";
            let mut f = File::open(&filename).expect("no file found"); //Len = 256
            f.read(&mut boot_rom).expect("buffer overflow");
            Self {
                //rom: [0; 16384],
                cart: Cartridge::new(cartridge),
                boot_rom: boot_rom,
                //exrom: ExROM::new(cart_contents.clone()),
                //external_ram: ExRam::new(cart_contents),
                work_ram: [0; 8192],
                high_ram: [0; 127],
                ppu: Screen::new(canvas),
                dma: DmaTransfer {
                    active: false,
                    addr_u: 0,
                    addr_l: 0,
                },
                wait: wait_ref,
                joypad: joypad,
                //serial: Serial { sb: 0, sc: 0 },
                timer: Timer {
                    divider: 0,
                    tima: 0,
                    tma: 0,
                    tac: 0,
                },
                iflag: 0,
                boot_control: 0,
                interrupts_enabled: 0, //Interrupts
                audio_controller: audio,
            }
        }
        pub fn grab_memory_8(&mut self, addr: u16) -> u8 {
            //info!("GET_MEMORY_8 ADDR{:#x}", addr);
            *self.wait.borrow_mut() += 2;
            self.memory_map(addr.into())
            //self.my_memory[addr as usize]
        }
        pub fn grab_memory_16(&mut self, addr: u16) -> u16 {
            //Well, that's one mystery solved.
            //#REMEMBER THIS IS IN LITTLE ENDIAN ORDER! THE BULLSHIT ONE! WE PUT THE SECOND BYTE FIRST
            *self.wait.borrow_mut() += 4;
            //info!("GET_MEMORY_16 ADDR{:#x}", addr);
            let high_byte = self.memory_map((addr + 1).into());
            let low_byte = self.memory_map((addr).into());
           
            (high_byte as u16) * (1 << 8) + (low_byte as u16)
        }
        pub fn set_memory_8(&mut self, addr: u16, value: u8) {
            self.memory_write((addr).into(), value);
        }
        pub fn set_memory_16(&mut self, addr: u16, value: u16) {
            self.memory_write((addr + 1).into(), (value >> 8) as u8);
            self.memory_write((addr).into(), (value % (1 << 8)) as u8);
        }
        pub fn on_clock(&mut self) {
            let video_interrupts = self.ppu.on_clock();
            if video_interrupts.0.is_some() {
                //LCDC
                if self.interrupts_enabled % 2 == 1 {
                    self.iflag |= 0x01;
                }
            }
            if video_interrupts.1.is_some() {
                //VBLANK
                if self.interrupts_enabled % 2 >> 1 == 1 {
                    self.iflag |= 0x02;
                }
            }
            if self.dma.active {
                self.dma()
            }
            if self.timer.on_clock().is_some(){
                if self.interrupts_enabled >> 2 % 2 == 1 {
                    self.iflag |= 0x04;
                }
            }   

            if self.joypad.on_clock().is_some() {
                if self.interrupts_enabled % 4 >> 1 == 1 {
                    self.iflag |= 0x10;
                }
            }
            self.audio_controller
                .handle_audio(self.timer.divider);
        }

        pub fn dma(&mut self) {
            let addr = 0x0000 + ((self.dma.addr_u) << 8) + self.dma.addr_l;
            let source = match addr {
                0x0000..=0x7FFF => self.cart.memory_map(addr.into()), //ROM
                0x8000..=0x9FFF => self.ppu.vram.memory_map((addr - 0x8000).into()), //VRAM
                0xA000..=0xBFFF => self.cart.memory_map(addr.into()), //External bus
                0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize],
                _ => unreachable!(),
            };
            self.ppu.oam.memory_write(self.dma.addr_l as usize, source);
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
