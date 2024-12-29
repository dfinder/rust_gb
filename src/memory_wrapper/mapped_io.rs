pub mod mapped_io {

    use std::{cell::RefCell, rc::Rc};

    use crate::{
        audio::audio_controller::AudioController, joypad::joypad::Joypad,
        memory_wrapper::memory_wrapper::AsMemory, screen::ppu::ppu::VideoController,
    };

    struct JoypadMIO {
        joypad_state: u8,
        buttons_ref: Rc<RefCell<Joypad>>,
    }
    impl AsByte for JoypadMIO {
        //Consider merging these concepts.
        fn read(&mut self) -> u8 {
            self.joypad_state = self
                .buttons_ref
                .borrow()
                .set_key_stroke_nibble(self.joypad_state);
            self.joypad_state
        }

        fn write(&mut self, val: u8) {
            self.joypad_state = val | 0xF0
        }
    }
    struct Serial {
        sb: u8, //Outside of scope :|
        sc: u8,
    }
    impl AsMemory for Serial{
        fn memory_map(&mut self, _addr: u16) -> u8 {
            0
        }
    
        fn memory_write(&mut self, _addr: u16, _val: u8) {
            ()
        }
    }
    trait AsByte {
        fn read(&mut self) -> u8;
        fn write(&mut self, val: u8);
    }
    struct Divider {
        internal_divider: u16, //Divider is secretly a 16 bit divider
    }
    impl AsByte for Divider {
        fn read(&mut self) -> u8 {
            return ((self.internal_divider & 0xF0) >> 8) as u8;
        }
        fn write(&mut self, _: u8) {
            self.internal_divider = 0;
        }
    }
    impl OnClock for Divider {
        fn on_clock(&mut self) {
            self.internal_divider += 1;
        }
    }
    struct Timer {
        divider: Divider, //Divider The div is the visible part of the system counter
        tima: u8,         //Timer counter.
        tma: u8,          //Timer reload.
        tac: u8,          //Timer control
    }
    impl AsMemory for Timer {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0 => self.divider.read(),
                1 => self.tima,
                2 => self.tma,
                3 => self.tac,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0 => self.divider.write(val),
                1 => todo!(),
                2 => todo!(),
                3 => todo!(),
                _ => unreachable!(),
            }
        }
    }

    impl OnClock for MappedIO {
        fn on_clock(&mut self) {
            //Timer control
            self.timer.divider.on_clock();
            let frequency = match self.timer.tac & 0x03 {
                0 => 8, //Every 256 m cycles
                1 => 2, //4 M cycles
                2 => 4, //16 m cycles
                3 => 6, //64 m cycles.
                _ => unreachable!(),
            };
            //If timer is enabled. If we hit 0 on the internal divider.
            if (self.timer.tac & 0x04 > 0)
                && ((self.timer.divider.internal_divider % (1 << frequency)) == 0)
            {
                let overflow: bool;
                (self.timer.tima, overflow) = self.timer.tima.overflowing_add(1);
                if overflow {
                    self.timer.tima = self.timer.tma;
                    self.iflag |= 0x04;
                }
            }
        }
    }
    pub trait OnClock {
        fn on_clock(&mut self) -> ();
    }
    struct InterruptFlag {
        inf: u8,
    }
    pub struct MappedIO {
        joypad: JoypadMIO, //FF00
        serial: Serial,    //FF01, FF02 [FF03 is unmapped]
        //div, //FF04, increments every clock cycle
        timer: Timer,
        iflag: u8,
        audio_controller: Rc<RefCell<AudioController>>,
        video_controller: Rc<RefCell<VideoController>>,
        boot_control: u8,
        ie: u8, //LCDControl,
        interrupt_flag:InterruptFlag,
    }

    impl MappedIO {
        pub fn new(
            joypad_ref: Rc<RefCell<Joypad>>,
            audio_con: Rc<RefCell<AudioController>>,
            video_con: Rc<RefCell<VideoController>>,
        ) -> Self {
            return Self {
                joypad: JoypadMIO {
                    joypad_state: 0,
                    buttons_ref: joypad_ref,
                },
                serial: Serial { sb: 0, sc: 0 },
                timer: Timer {
                    divider: Divider {
                        internal_divider: 0,
                    },
                    tima: 0,
                    tma: 0,
                    tac: 0,
                },
                iflag: 0,
                boot_control: 0,
                ie: 0,
                audio_controller: audio_con,
                video_controller: video_con,
                interrupt_flag:InterruptFlag{inf:0}
            };
        }
    }
    impl AsMemory for MappedIO {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                //todo!()
                0x00 => self.joypad.read(),
                0x01 => 0, //Serial
                0x02 => 0,
                0x03 => 0,
                0x04..=0x07 => self.timer.memory_map(addr - 0x0004),
                0x0f => 0xE0 | self.iflag,
                0x10..0x26 => self.audio_controller.borrow_mut().memory_map(addr-0x0010),
                0x40..=0x4b => self.video_controller.borrow_mut().memory_map(addr-0x0040),
                //=>
                0x50=>self.boot_control,
                0xff => self.ie,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            //Begins at FF00
            match addr {
                0x0000 => self.joypad.write(val),
                0x0001 => todo!(),
                0x0002 => todo!(), //Serial
                0x0003 => todo!(), //Unmapped
                0x0004..=0x0007 => self.timer.memory_write(addr - 0x0004, val),
                0x000f => self.iflag = 0xE0 | val,
                0x0010..=0x003f => self
                    .audio_controller
                    .borrow_mut()
                    .memory_write(addr - 0x0010, val),

                0x40..=0x4b => self.video_controller.borrow_mut().memory_write(addr-0x0040,val),
                0x50=>self.boot_control=val,
                0xff => self.ie=val,
                _ => unreachable!(),
            }
        }
    }
}
