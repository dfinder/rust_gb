pub mod audio_controller{
    use std::ops::{Index, IndexMut};

    struct AudioController{
      audio_channel1: AudioChannel1, //FF10
      audio_channel2: AudioChannel2,//FF16
      audio_channel3: AudioChannel3, //FF1A
      audio_channel4: AudioChannel4, //FF20
      audio_output: u8, // FF24: 
      audio_channel_mapping:u8, //FF25 //Controls panning
      audio_channel_control:u8,//FF26 //Audio master control
      wave_pattern:u8 //FF3F
    }
    struct AudioChannel1{
        sweep:u8,
        sound_length: u8,
        envelope: u8, 
        frequency: u8, 
        control: u8
        
    }
    struct AudioChannel2{
        unmapped: u8,
        sound_length: u8,
        envelope: u8, 
        frequency: u8, 
        control: u8,
    }
    struct AudioChannel3{
        enable:u8,
        sound_length: u8,
        envelope: u8, 
        frequency: u8, 
        control: u8,
    }
    struct AudioChannel4{
        sound_length: u8,
        volume: u8,
        frequency: u8, 
        control: u8,
    }
    impl Index<u16> for AudioController{
        type Output = u8;
        fn index(&self, index: u16) -> &Self::Output {
            todo!()
        }
    }
    impl IndexMut<u16> for AudioController{
        fn index_mut(&mut self, index: u16) -> &mut Self::Output {
            todo!()
        }
    }
    struct SoundState{
        //Quadrangular wave sweep and ev
        //Quadragular wave with envelope

    }
}