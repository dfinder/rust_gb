mod mapped_io{
    struct joypad {

    }
    struct serial{

    }
    trait AsByte{
        fn read(&self)->u8;
        fn write(&self,val:u8);
    }
    struct divider{
        divider:u8, //FFO4
    }
    impl OnClock for divider{
        fn on_clock(&self){
            self.divider+=1;
        }
    }
    //#[repr(C)]
    //#[repr(packed)]
    struct timer{
        
        tmrr:u8,
        tma:u8, //Timer reload. 
        tac:u8, //Timer control
    }
    impl OnClock for mapped_io{
        fn on_clock(&self){
            let frequency = match self.timer.tac & 0x03{
                0 => 1,
                1 => 64,
                2 => 16,
                3 => 4,
            };
            //So our options are 1/cycle, 4/cycle, 16/cycle, and 64/cycle
            if self.timer.tac & 0x04{
                let mut overflow = false;
                (self.tmr, overflow) =  self.timer.overflowing_add(frequency);
                if overflow{
                    self.tmr = tma;
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
    struct interrupt_flag{
        inf:u8
    }
    struct mapped_io{
        joypad, //FF00
        serial, //FF01, FF02 [FF03 is unmapped]
        div, //FF04, increments every clock cycle
        tmr:timer,
        iflag:u8,
        LCDControl,
        


    }

    impl mapped_io{
        fn new()->Self{

        }
        
    }
    
}