//We have 4 sound channels:
//Square, sweep
//Square
//Wave
//Noise
pub mod audio_impl {
    struct AudioSystem {}
    struct SquareWave {}
    struct SquareSweep {}
    struct Wave {}
    struct Noise {}
    impl AudioSystem {
        fn init() -> Self {
            return Self {};
        }
        fn set_channels() {}
        fn handle_audio() {}
    }
}
pub mod audio_controller {

    use crate::memory_wrapper::memory_wrapper::AsMemory;

    pub struct AudioController {
        audio_channel1: AudioChannel1, //FF10
        audio_channel2: AudioChannel2, //FF16
        audio_channel3: AudioChannel3, //FF1A
        audio_channel4: AudioChannel4, //FF20
        audio_output: u8,              // FF24:
        audio_channel_mapping: u8,     //FF25 //Controls panning
        audio_channel_control: u8,     //FF26 //Audio master control
        wave_pattern: u8,              //FF3F
    }
    impl AudioController {
        pub fn new() -> Self {
            return Self {
                audio_channel1: AudioChannel1::new(),
                audio_channel2: AudioChannel2::new(),
                audio_channel3: AudioChannel3::new(),
                audio_channel4: AudioChannel4::new(),
                audio_output: 0,
                audio_channel_mapping: 0,
                audio_channel_control: 0,
                wave_pattern: 0,
            };
        }
    }
    impl AsMemory for AudioController {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000..=0x0005 => self.audio_channel1.memory_map(addr),
                0x0006..=0x0009 => self.audio_channel2.memory_map(addr - 0x006),
                0x000A..=0x000F => self.audio_channel3.memory_map(addr - 0x00A),
                0x0010..=0x0013 => self.audio_channel4.memory_map(addr - 0x010),
                0x0014 => self.audio_output,
                0x0015 => self.audio_channel_mapping,
                0x0016 => self.audio_channel_control,
                0x0017..=0x001f => unreachable!(),
                0x0020 => self.wave_pattern,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000..=0x0005 => self.audio_channel1.memory_write(addr, val),
                0x0006..=0x0009 => self.audio_channel2.memory_write(addr - 0x006, val),
                0x000A..=0x000F => self.audio_channel3.memory_write(addr - 0x00A, val),
                0x0010..=0x0013 => self.audio_channel4.memory_write(addr - 0x010, val),
                0x0014 => self.audio_output = val,
                0x0015 => self.audio_channel_mapping = val,
                0x0016 => self.audio_channel_control = val,
                0x0017..=0x001f => unreachable!(),
                0x0020 => self.wave_pattern = val,
                _ => unreachable!(),
            }
        }
    }
    struct AudioChannel1 {
        sweep: u8,
        sound_length: u8,
        envelope: u8,
        frequency: u8,
        control: u8,
        unmapped: u8,
    }
    impl AudioChannel1 {
        fn new() -> Self {
            return Self {
                sweep: 0,
                sound_length: 0,
                envelope: 0,
                frequency: 0,
                control: 0,
                unmapped: 0,
            };
        }
    }
    impl AsMemory for AudioChannel1 {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.sweep,
                0x0001 => self.sound_length,
                0x0002 => self.envelope,
                0x0003 => self.frequency,
                0x0004 => self.control,
                0x0005 => self.unmapped,
                _ => unreachable!(),
            }
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            match addr {
                0x0000 => self.sweep = val,
                0x0001 => self.sound_length = val,
                0x0002 => self.envelope = val,
                0x0003 => self.frequency = val,
                0x0004 => self.control = val,
                0x0005 => self.unmapped = val,
                _ => unreachable!(),
            }
        }
    }
    struct AudioChannel2 {
        sound_length: u8,
        envelope: u8,
        frequency: u8,
        control: u8,
    }
    impl AudioChannel2 {
        fn new() -> Self {
            return Self {
                sound_length: 0,
                envelope: 0,
                frequency: 0,
                control: 0,
            };
        }
    }
    impl AsMemory for AudioChannel2 {
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
    struct AudioChannel3 {
        enable: u8,
        sound_length: u8,
        envelope: u8,
        frequency: u8,
        control: u8,
        unmapped: u8,
    }

    impl AudioChannel3 {
        fn new() -> Self {
            return Self {
                enable: 0,
                sound_length: 0,
                envelope: 0,
                frequency: 0,
                control: 0,
                unmapped: 0,
            };
        }
    }
    impl AsMemory for AudioChannel3 {
        fn memory_map(&mut self, addr: u16) -> u8 {
            match addr {
                0x0000 => self.enable,
                0x0001 => self.sound_length,
                0x0002 => self.envelope,
                0x0003 => self.frequency,
                0x0004 => self.control,
                0x0005 => self.unmapped,
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
                0x0005 => self.unmapped = val,
                _ => unreachable!(),
            }
        }
    }

    struct AudioChannel4 {
        sound_length: u8,
        volume: u8,
        frequency: u8,
        control: u8,
    }
    impl AudioChannel4 {
        fn new() -> Self {
            return Self {
                sound_length: 0,
                volume: 0,
                frequency: 0,
                control: 0,
            };
        }
    }
    impl AsMemory for AudioChannel4 {
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
    struct SoundState {
        //Quadrangular wave sweep and ev
        //Quadragular wave with envelope
    }
}
