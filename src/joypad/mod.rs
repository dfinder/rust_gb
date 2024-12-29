pub mod joypad {
    use winit::{
        event::{DeviceId, KeyEvent},
        keyboard::{KeyCode, PhysicalKey},
    };

    use crate::cpu::cpu::CpuStruct;
    use crate::interrupt::interrupt::Interrupt;
    #[derive(Clone, Copy)]
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
        p_key: KeyCode,
        state: bool,
        //time: u32
    }
    #[derive(Clone, Copy)]
    pub struct Joypad {
        mapping: [KeyWrapper; 8],
    }

    impl Joypad {
        pub fn new(map: [KeyCode; 8]) -> Self {
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
                mapping: [start, select, b, a, down, up, left, right],
            };
        }
        pub fn process_keystrokes(
            &mut self,
            cpu: &mut CpuStruct,
            _d_id: DeviceId,
            key_event: KeyEvent,
            synthetic: bool,
        ) {
            for key_wrapper in self.mapping.iter_mut() {
                //match key_event
                if synthetic == false && key_event.repeat == false {
                    match key_event.physical_key {
                        PhysicalKey::Code(key_press) => {
                            if key_press == key_wrapper.p_key {
                                key_wrapper.state = !key_event.state.is_pressed();
                                if !key_event.state.is_pressed() {
                                    //cpu.unstop();
                                    cpu.interrupt(Interrupt::Input);
                                }
                            }
                        }
                        PhysicalKey::Unidentified(_) => (),
                    }
                }
            }
        }
        pub fn set_key_stroke_nibble(self, current_val: u8) -> u8 {
            let mut initial_index: usize = 0;
            let mut nibble: u8 = current_val;
            if current_val >= 0x30 {
                //Figure out what to do if both are set.
                return current_val | 0x0F;
            }
            if (current_val & 0x10) > 0 {
                //Fetch DPad on bit 4.
                initial_index = 4
            } //
            nibble |= 0x0F;
            for i in 0..4 {
                nibble &= (!self.mapping[i + initial_index].state as u8) * (1 << (4 - i));
            }
            return nibble;
        }
        //fn hit_list()This also occurs during a speed switch. (TODO: how is it affected by the wait after a speed switch?)
    }
}
