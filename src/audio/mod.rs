

//We have 4 sound channels:
//Square, sweep
//Square
//Wave
//Noise

pub mod audio_controller {

    use sdl2::audio::{AudioQueue, AudioSpecDesired};

    use crate::memory::memory_wrapper::AsMemory;

    pub struct AudioController {
        pub audio_channel1: SquareSweep,  //FF10
        pub audio_channel2: SquareWave,   //FF16
        pub audio_channel3: Wave, //FF1A
        pub audio_channel4: Noise,        //FF20
        pub audio_volume: u8,             //FF24
        pub audio_panning: u8,    //FF25 //Controls panning
        pub audio_master_control: u8,    //FF26 //Audio master control
        pub wave_pattern: u8,             //FF3F
    }
    impl AudioController {
        pub fn new(sdl_audio: sdl2::AudioSubsystem) -> Self {
            
            let desired_spec = AudioSpecDesired {
                freq: Some(44100),
                channels: Some(2), // mono
                samples: None,     // default sample size
            };
            let channel_1: sdl2::audio::AudioQueue<u8> = sdl_audio.open_queue(None, &desired_spec).expect("failed to process chan 1");
            let channel_2: sdl2::audio::AudioQueue<u8> = sdl_audio.open_queue(None, &desired_spec).expect("failed to process chan 2");
            let channel_3: sdl2::audio::AudioQueue<u8> = sdl_audio.open_queue(None, &desired_spec).expect("failed to process chan 3");
            let channel_4: sdl2::audio::AudioQueue<u8> = sdl_audio.open_queue(None, &desired_spec).expect("failed to process chan 4");
            Self {
                audio_channel1: SquareSweep::new(channel_1),
                audio_channel2: SquareWave::new(channel_2),
                audio_channel3: Wave::new(channel_3),
                audio_channel4: Noise::new(channel_4),
                audio_volume: 0,
                audio_panning: 0,
                audio_master_control: 0,
                wave_pattern: 0,
            }
        }
        pub fn initialize_audio(&mut self){
                   
        }

        pub fn handle_audio(&mut self, current_timer: u16) {
            let _ = current_timer;
            self.audio_channel1.process_audio(self.audio_master_control, self.audio_panning, self.audio_volume);
        }
        fn audio_state(&mut self)->u8{
            self.audio_master_control & 0x80 + 8 * (self.audio_channel4.is_on()) + 4 * (self.audio_channel3.is_on()) + 2* (self.audio_channel2.is_on())+(self.audio_channel1.is_on())
        }
    }
    trait AudioChannel {

        fn process_audio(&mut self,audio_control:u8,panning:u8,volume:u8);
        
    }
    
    impl AsMemory for AudioController {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000..=0x0005 => self.audio_channel1.memory_map(addr),
                0x0006..=0x0009 => self.audio_channel2.memory_map(addr - 0x006),
                0x000A..=0x000F => self.audio_channel3.memory_map(addr - 0x00A),
                0x0010..=0x0013 => self.audio_channel4.memory_map(addr - 0x010),
                0x0014 => self.audio_volume,
                0x0015 => self.audio_panning,
                0x0016 => self.audio_state(), //REWRITE: this re
                0x0017..=0x001f => unreachable!(),
                0x0020 => self.wave_pattern,
                _ => unreachable!(),
            }
        }
    
        fn memory_write(&mut self, addr: u16, val: u8) {
            if self.audio_master_control &0x80 ==0 {
                match addr{

                    0x0016 => self.audio_master_control = val&0x80,
                    _ => ()
                }
            }
            match addr {
                0x0000..=0x0005 => self.audio_channel1.memory_write(addr, val),
                0x0006..=0x0009 => self.audio_channel2.memory_write(addr - 0x006, val),
                0x000A..=0x000F => self.audio_channel3.memory_write(addr - 0x00A, val),
                0x0010..=0x0013 => self.audio_channel4.memory_write(addr - 0x010, val),
                0x0014 => self.audio_volume = val,
                0x0015 => self.audio_panning = val,
                0x0016 => self.audio_master_control = val&0x80,
                0x0017..=0x001f => unreachable!(),
                0x0020..=0x002F => self.audio_channel3.wave_pattern_ram[(addr-0x0020) as usize] = val,
                _ => unreachable!(),
            }
        }
    }

    pub struct SquareSweep {
        sweep: u8,
        sound_length: u8,
        envelope: u8,
        frequency: u8,
        control: u8,
        queue: AudioQueue<u8>,
        temp_sweep: u8,
        period:u8

    }
    impl SquareSweep {
        fn new(queue: AudioQueue<u8>) -> Self {
            return Self{

                sweep: 0,
                sound_length: 0,
                envelope:0,
                frequency: 0,
                control: 0,
                queue,
                temp_sweep:0,
                period:0


            }
        }
        fn is_on(&mut self)->u8{
            true as u8
        }
    }
    impl AudioChannel for SquareSweep{
        fn process_audio(&mut self,audio_control:u8,panning:u8,volume:u8) {
            if audio_control>>7 %2  == 0{
                self.queue.clear();
                self.sweep=0;
                self.sound_length=0;
                self.envelope=0;
                self.frequency=0;
                self.control=0;
            }
            let left_pan = panning >> 4 % 2 == 1;
            let right_pan = panning % 2 == 1;
            let left_volume = volume >>4 %8+1;
            let right_volume = volume % 8+1;
            let pace = self.sweep>>4 % 8;
            let direction = self.sweep>4 % 2;
            let individual_step = self.sweep % 8 ;
            let wave_duty = self.sound_length >> 6 % 4;
            let initial_timer = self.sound_length % 1>> 5;
            let initial_volume = self.envelope >> 4;
            let env_dir = self.envelope >> 3 %2;
            let sweep_pace = self.envelope %8;
            let period:u16 = (self.control % 8)as u16 + self.frequency as u16;
            let length_enable = self.control>>6 %2 == 1;
            let trigger = self.control >> 7 == 1;
            if trigger{

            }
            if length_enable{

            }


        }
        //I run every 1/64 of a seocnd?
    }
    impl AsMemory for SquareSweep {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.sweep,
                0x0001 => self.sound_length&0xC0, //Bits 5-0 are write only
                0x0002 => self.envelope,
                0x0003 => 0,
                0x0004 => self.control&0x47,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000 => self.temp_sweep = val,
                0x0001 => self.sound_length = val,
                0x0002 => self.envelope = val,
                0x0003 => self.frequency = val,
                0x0004 => self.control = val&0xc7,
                _ => unreachable!(),
            }
        }
    }

    pub struct SquareWave {
        sound_length: u8,
        envelope: u8,
        frequency: u8,
        control: u8,

        queue: AudioQueue<u8>
    }
    impl SquareWave {
        fn new(
            queue: AudioQueue<u8>) -> Self {
            return Self {
                sound_length: 0,
                envelope: 0,
                frequency: 0,
                control: 0,
                queue
            };
        }

        fn is_on(&mut self)->u8{
            true as u8
        }
    }
    impl AsMemory for SquareWave {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.sound_length,
                0x0001 => self.envelope,
                0x0002 => self.frequency,
                0x0003 => self.control,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000 => self.sound_length = val,
                0x0001 => self.envelope = val,
                0x0002 => self.frequency = val,
                0x0003 => self.control = val,
                _ => unreachable!(),
            }
        }
    }

    pub struct Wave {
        enable: u8,
        sound_length: u8,
        envelope: u8,
        frequency: u8,
        control: u8,
        wave_pattern_ram:[u8;16],
        queue: AudioQueue<u8>
    }

    impl Wave {
        fn new(
            queue: AudioQueue<u8>) -> Self {
            return Self {
                enable: 0,
                sound_length: 0,
                envelope: 0,
                frequency: 0,
                control: 0,
                wave_pattern_ram:[0;16],
                queue
            };
        }
        
        fn is_on(&mut self)->u8{
            true as u8
        }
    }
    impl AsMemory for Wave {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.enable,
                0x0001 => self.sound_length,
                0x0002 => self.envelope,
                0x0003 => self.frequency,
                0x0004 => self.control,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000 => self.enable = val,
                0x0001 => self.sound_length = val,
                0x0002 => self.envelope = val,
                0x0003 => self.frequency = val,
                0x0004 => self.control = val,
                _ => unreachable!(),
            }
        }
    }

    pub struct Noise {
        sound_length: u8,
        volume: u8,
        frequency: u8,
        control: u8,
        queue: AudioQueue<u8>
    }
    impl Noise {
        fn new(
            queue: AudioQueue<u8>) -> Self {
            return Self {
                sound_length: 0,
                volume: 0,
                frequency: 0,
                control: 0,
                queue
            };
        }
        fn is_on(&mut self)->u8{
            true as u8
        }
    }
    impl AsMemory for Noise {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.sound_length,
                0x0001 => self.volume,
                0x0002 => self.frequency,
                0x0003 => self.control,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000 => self.sound_length = val,
                0x0001 => self.volume = val,
                0x0002 => self.frequency = val,
                0x0003 => self.control = val,
                _ => unreachable!(),
            }
        }
    }
}
