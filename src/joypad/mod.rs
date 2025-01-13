pub mod joypad {

    use std::{cell::RefCell, rc::Rc};

    use sdl2::{keyboard::Scancode, EventPump};

    use crate::cpu::interrupt::interrupt::Interrupt;
 

    #[derive(Clone, Copy, Debug
    )]
    enum GBKey {
        Start,
        Select,
        B,
        A,
        Down,
        Up,
        Left,
        Right,
    }
    #[derive(Clone, Copy)]
    struct KeyWrapper {
        v_key: GBKey,
        p_key: Scancode,
        state: bool,
        //time: u32
    }
    pub struct Joypad {
        mapping: [KeyWrapper; 8],
        sdl_handler:Rc<RefCell<EventPump>>,
        joypad_state:u8
    }
    impl Joypad {
        pub fn new(map: [Scancode; 8],sdl_handler: Rc<RefCell<EventPump>>) -> Self {
            let start: KeyWrapper = KeyWrapper {
                v_key: GBKey::Start,
                p_key: map[0],
                state: false,
            };
            let select: KeyWrapper = KeyWrapper {
                v_key: GBKey::Select,
                p_key: map[1],
                state: false,
            };

            let b: KeyWrapper = KeyWrapper {
                v_key: GBKey::B,
                p_key: map[2],
                state: false,
            };
            let a: KeyWrapper = KeyWrapper {
                v_key: GBKey::A,
                p_key: map[3],
                state: false,
            };
            let down: KeyWrapper = KeyWrapper {
                v_key: GBKey::Down,
                p_key: map[4],
                state: false,
            };
            let up: KeyWrapper = KeyWrapper {
                v_key: GBKey::Up,
                p_key: map[5],
                state: false,
            };
            let left: KeyWrapper = KeyWrapper {
                v_key: GBKey::Left,
                p_key: map[6],
                state: false,
            };
            let right: KeyWrapper = KeyWrapper {
                v_key: GBKey::Right,
                p_key: map[7],
                state: false,
            };
            return Joypad {
                mapping: [start, select, b, a, down, up, left, right],sdl_handler,joypad_state:0
            };
        }
        /* pub fn process_keystrokes(
            &mut self,
            cpu: &mut CpuStruct,
            key_event: Option<Keycode>,
            orientation: bool,
        ) {
            //println!("We process a key event {:?}",key_event.unwrap_or(Keycode::KP_000));
                match key_event{
                    Some(key_press) => {
                        for mut button in self.mapping{
                            if button.p_key==key_press{
                                button.state = orientation;
                                if orientation{
                                    println!("We tell the CPU that there's a processor interrupt for key {:?}",button.v_key);
                                    cpu.interrupt(Interrupt::Input);
                                }
                            }
                        }
                    },
                    None => ()
                }
            
        } */
        pub fn on_clock(&mut self)->Option<Interrupt>{
            let mut ret = None;
            let handler = self.sdl_handler.borrow_mut();
            let keyboard_state = handler.keyboard_state();
            for mut i in self.mapping{
                let is_pressed = keyboard_state.is_scancode_pressed(i.p_key);
                if ret.is_none() && i.state != is_pressed {
                    ret = Some(Interrupt::Input);
                }
                i.state = is_pressed

            }
            ret 

        }
        pub fn set_key_stroke_nibble(&mut self) -> u8 {
            self.joypad_state=self.joypad_state | 0x0F; //We clear the lower nibble
            if self.joypad_state < 0x30 {
                let mut initial_index: usize = 0;
                if (self.joypad_state & 0x10) > 0 {
                    //Fetch DPad on bit 4.
                    initial_index = 4
                } 
                for i in 0..4 {
                    self.joypad_state &= (!self.mapping[i + initial_index].state as u8) * (1 << (4 - i));
                }
            }
            return self.joypad_state;
        }
        pub fn read(&mut self) -> u8 {
            self.set_key_stroke_nibble()
        }
        pub fn write(&mut self, val: u8) {
            self.joypad_state = val & 0x30
        }
        //fn hit_list()This also occurs during a speed switch. (TODO: how is it affected by the wait after a speed switch?)
    }
}
