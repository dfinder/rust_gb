pub mod mapped_io{

    use std::ops::Index;

    struct Joypad {
        joypad_state: u8
    }
    impl AsByte for Joypad{
        fn read(&mut self)->u8 {
            todo!()
        }
    
        fn write(&mut self,val:u8) {
            todo!()
        }
    }
    struct Serial{
        sb:u8, //Outside of scope :|
        sc:u8
    }
    trait AsByte{
        fn read(&mut self)->u8;
        fn write(&mut self,val:u8);
    }
    struct Divider{
        internal_divider: u16 //Divider is secretly a 16 bit divider
    }
    impl AsByte for Divider{ 
        fn read(&mut self)->u8{
            return ((self.internal_divider&0xF0)>>8) as u8 
        }
        fn write(&mut self,_:u8){
            self.internal_divider=0;
        }
    }
    impl OnClock for Divider{
        fn on_clock(&mut self){
            self.internal_divider+=1;
        }
    }
    struct Timer{
        divider:Divider, //Divider The did is the visible part of the system counter
        tima:u8, //Timer counter. 
        tma:u8, //Timer reload. 
        tac:u8, //Timer control
    }
    //
    impl Index<u16> for Timer{
        type Output=u8;
        
        fn index(&self, index: u16) -> &Self::Output { //This structure doesn't quite work the way we want it to....
            match index{
                0x0 => todo!(),//self.divider, //FF04
                0x1 => todo!(),//self.tima,
                0x2 => todo!(),//self.tma, 
                0x3 => todo!(),//self.tac,
                _ => unreachable!()
            }
        }
    }
    impl OnClock for MappedIO{
        fn on_clock(&mut self){ //Timer control
            self.timer.divider.on_clock();
            let frequency = match self.timer.tac & 0x03{
                0 => 8, //Every 256 m cycles 
                1 => 2, //4 M cycles 
                2 => 4, //16 m cycles 
                3 => 6, //64 m cycles.
                _ => unreachable!()
            };
            //If timer is enabled. If we hit 0 on the internal divider.
            if (self.timer.tac & 0x04 > 0) && ((self.timer.divider.internal_divider %(1<<frequency)) == 0){
                let overflow:bool;
                (self.timer.tima, overflow) =  self.timer.tima.overflowing_add(1);
                if overflow{
                    self.timer.tima=self.timer.tma;
                    self.iflag |= 0x04;
                }
            }
        }
    }
    /**impl AsByte for divider{
        pub trait AsByte{
            fn read(&self)->u8;
            fn write(&self,val:u8);
        }
    }**/
    pub trait OnClock{
        fn on_clock(&mut self)->();
    }
    struct InterruptFlag{
        inf:u8
    }
    pub struct MappedIO{
        joypad:Joypad, //FF00
        serial:Serial, //FF01, FF02 [FF03 is unmapped]
        //div, //FF04, increments every clock cycle
        timer:Timer,
        iflag:u8,
        //audio_controler:AudioController,
        //video_controller:VideoController,
        boot_control: u8,
        IE: u8
        //LCDControl,
        


    }


    impl MappedIO{
        fn new()->Self{
            return Self{ joypad: Joypad{joypad_state:0}, serial: (), timer: Timer{divider:Divider{internal_divider:0},tima:0,tma:0,tac:0}, iflag: 0, boot_control: 0, IE: 0 }

        }
        
    }
    impl Index<u16> for MappedIO{
        type Output=u8;
        
        fn index(&self, index: u16) -> &Self::Output {
            //todo!()
            match addr{
                0x0000 => self.joypad.read(),
                0x0001 => todo!(),
                0x0002 => todo!(), //Serial
                0x0003 => todo!(),//Unmapped
                0x0004..=0x0007 => &self.timer[index-0x0004],
                0x000f => 0xE0 | self.iflag,
                0x0010 => todo!(),
                //0x0040 => self.lcd_control,
                //=> 
                _ => unreachable!()
            }
        }
    }
    
}