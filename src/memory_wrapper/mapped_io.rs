pub mod mapped_io{
    use crate::memory_wrapper::memory_wrapper::AsMemory;

    struct Joypad {

    }
    impl AsByte for Joypad{
        fn read(&self)->u8 {
            todo!()
        }
    
        fn write(&self,val:u8) {
            todo!()
        }
    }
    struct Serial{


    }
    trait AsByte{
        fn read(&self)->u8;
        fn write(&self,val:u8);
    }
    struct Divider{
        internal_divider: u16 //Divider is secretly a 16 bit divider
    }
    impl AsByte for Divider{ 
        fn read(&self)->u8{
            return ((self.internal_divider&0xF0)>>8) as u8 
        }
        fn write(&self,val:u8){
            self.internal_divider=0;
        }
    }
    impl OnClock for Divider{
        fn on_clock(&self){
            self.internal_divider+=1;
        }
    }
    struct Timer{
        divider:Divider, //Divider The did is the visible part of the system counter
        tima:u8, //Timer counter. 
        tma:u8, //Timer reload. 
        tac:u8, //Timer control
    }
    impl OnClock for MappedIO{
        fn on_clock(&self){ //Timer control
            self.timer.divider.on_clock();
            let frequency = match self.timer.tac & 0x03{
                0 => 8, //Every 256 m cycles 
                1 => 2, //4 M cycles 
                2 => 4, //16 m cycles 
                3 => 6, //64 m cycles.
            };
            //If timer is enabled. If we hit 0 on the internal divider.
            if (self.timer.tac & 0x04 > 0) && ((self.timer.divider.internal_divider %(1<<frequency)) == 0){
                let mut overflow = false;
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
        fn on_clock(&self)->();
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
        //LCDControl,
        


    }


    impl MappedIO{
        fn new()->Self{
            

        }
        
    }
    impl AsMemory for MappedIO{
        fn memory_map(&mut self,addr:u16)->u8 {
            match addr{
                0x0000 => self.joypad.read(),
                0x0001 => todo!(),
                0x0002 => todo!(), //Serial
                0x0003 => todo!(),//Unmapped
                
                0x0010 => 
                _=> 
            }
        }
    
        fn memory_write(&mut self,addr:u16,val:u8) {
            todo!()
        }
    }
    
}