pub mod joypad{
    use winit::{event::{DeviceId, KeyEvent}, keyboard::{KeyCode, PhysicalKey}};

    use crate::cpu::cpu::CpuStruct;
    enum GBKey{
        A,B,Up,Down,Left,Right,Start,Select
    }
    struct KeyWrapper{
        v_key: GBKey,
        p_key: KeyCode,
        state: bool,
        //time: u32
    }
    pub struct Joypad{
        mapping:[KeyWrapper;8]
    }
    impl Joypad {
        pub fn new(map:[KeyCode;8]) -> Self{
            let a:KeyWrapper = KeyWrapper{v_key: GBKey::A,p_key:map[0],state:false};
            let b:KeyWrapper = KeyWrapper{v_key: GBKey::B,p_key:map[1],state:false};
            let up:KeyWrapper = KeyWrapper{v_key: GBKey::Up,p_key:map[2],state:false};
            let down:KeyWrapper =KeyWrapper{v_key: GBKey::Down,p_key:map[3],state:false};
            let left:KeyWrapper = KeyWrapper{v_key: GBKey::Left,p_key:map[4],state:false};
            let right:KeyWrapper = KeyWrapper{v_key: GBKey::Right,p_key:map[5],state:false}; 
            let start:KeyWrapper = KeyWrapper{v_key: GBKey::Start,p_key:map[6],state:false}; 
            let select:KeyWrapper = KeyWrapper{v_key: GBKey::Select,p_key:map[7],state:false};
            return Joypad{

                mapping:[a,b,up,down,left,right,start,select]

            }
        }
        pub fn process_keystrokes(&mut self, d_id: DeviceId, key_event: KeyEvent, synthetic: bool){
          for (index,key_wrapper ) in self.mapping.iter().enumerate(){
            //match key_event
            if synthetic == false && key_event.repeat==false {
                match key_event.physical_key{
                    PhysicalKey::Code(key_press) => {
                        if key_press == key_wrapper.p_key{
                            key_wrapper.state = !key_event.state.is_pressed();
                        }
                    }
                    PhysicalKey::Unidentified(_) => todo!(),
                }
            }
          }
        }
        pub fn set_key_stroke_nibble(&mut self, current_val:u8,)->u8{
            

        }
        //fn hit_list()
    }
}