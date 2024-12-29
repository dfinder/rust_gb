pub mod mbc3 {
    use crate::cartridge::mbc::mbc::{ram_size, rom_size, Bank, Mbc};
    use core::time;
    use std::{
        cmp::max,
        sync::{Arc, Mutex},
        thread::{self},
    };
    pub struct Mbc3 {
        rom_bank_num: usize,
        ram_bank_num: usize,
        rom: Vec<Bank>,
        ram: Vec<Bank>,
        ram_enable: bool,
        clock_mode: u8,
        tmp_clock: RTC,
        clock: Arc<Mutex<RTC>>,
        latched: u8,
    }
    #[derive(Clone, Copy)]
    pub struct RTC {
        seconds: u8,
        minutes: u8,
        hours: u8,
        days_l: u8,
        days_h: u8,
    }
    impl RTC {
        fn new() -> Self {
            return Self {
                seconds: 0,
                minutes: 0,
                hours: 0,
                days_l: 0,
                days_h: 0,
            };
        }
        fn on_sec(&mut self) {
            if self.days_h & 0x40 == 0 {
                self.seconds += 1;
                if self.seconds > 59 {
                    self.seconds = 0;
                    self.minutes += 1;
                    if self.minutes > 59 {
                        self.hours += 1;
                        if self.hours > 23 {
                            let result = self.days_l.checked_add(1);
                            match result {
                                Some(x) => self.days_l = x,
                                None => {
                                    self.days_l = 0;
                                    if self.days_h % 2 == 0 {
                                        self.days_h |= 1
                                    } else {
                                        self.days_h |= 0x80
                                    }
                                }
                            };
                        }
                    }
                }
            }
        }
    }
    impl Mbc for Mbc3 {
        fn new(cart: Vec<u8>) -> Self
        where
            Self: Sized,
        {
            let rom: Vec<Bank> = vec![[0; 16384]; rom_size(cart[0x0148])];
            let ram: Vec<Bank> = vec![[0; 16384]; ram_size(cart[0x0149])];
            let clock: RTC = RTC::new();
            let clock_mutex = Mutex::new(clock);
            let local_rc = Arc::new(clock_mutex);
            let real_rc = local_rc.clone();
            thread::spawn(move || loop {
                thread::sleep(time::Duration::from_secs(1));
                let clock = real_rc.lock();
                match clock {
                    Ok(mut mutex_clock) => mutex_clock.on_sec(),
                    Err(_) => todo!(),
                }
            });
            return Self {
                rom_bank_num: 0,
                ram_bank_num: 0,
                rom: rom,
                ram: ram,
                ram_enable: false,
                clock_mode: 0,
                tmp_clock: RTC::new(),
                clock: local_rc.clone(),
                latched: 0,
            };
        }
        fn rom_read(&mut self, addr: u16) -> u8 {
            match addr {
                0..=0x3FFF => self.rom[0][addr as usize],
                0x4000..=0x7fff => self.rom[self.rom_bank_num][addr as usize],
                _ => unreachable!(),
            }
        }

        fn rom_write(&mut self, addr: u16, val: u8) {
            match addr {
                0..=0x1fff => self.ram_enable = val == 0xA,
                0x2000..=0x3fff => {
                    self.rom_bank_num =
                        max(1, (val & 0x1F) as usize + (self.ram_bank_num << 5)) % self.ram.len()
                }
                0x4000..=0x5fff => {
                    if (0..=3).contains(&val) {
                        self.clock_mode = 0;
                        self.ram_bank_num = val as usize;
                    }
                    if (0x08..=0x0C).contains(&val) {
                        self.clock_mode = val;
                    }
                }
                0x6000..=0x7fff => {
                    if val == 0 && self.latched == 0 {
                        self.latched = 1;
                    }
                    if val == 1 && self.latched == 1 {
                        self.latched = 0;
                        let x = self.clock.lock();
                        match x {
                            Ok(inner_clock) => self.tmp_clock = inner_clock.clone(),
                            Err(_) => todo!(),
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        fn ram_read(&mut self, addr: u16) -> u8 {
            //So this is the space A000->BFFF. We only go away from this if we turn on the bank mode.
            if !self.ram_enable {
                return 0xFF;
            } else {
                if self.clock_mode > 8 {
                    return match self.clock_mode {
                        8 => self.tmp_clock.seconds,
                        9 => self.tmp_clock.minutes,
                        0xa => self.tmp_clock.hours,
                        0xb => self.tmp_clock.days_l,
                        0xc => self.tmp_clock.days_h,
                        _ => unreachable!(),
                    };
                }
                return self.ram[self.ram_bank_num][addr as usize];
            }
        }
        fn ram_write(&mut self, addr: u16, val: u8) {
            if self.ram_enable {
                if self.clock_mode > 8 {
                    let clock_result = self.clock.lock();
                    match clock_result {
                        Ok(mut inner_clock) => match self.clock_mode {
                            8 => inner_clock.seconds = val % 60,
                            9 => inner_clock.minutes = val % 60,
                            0xa => inner_clock.hours = val % 24,
                            0xb => inner_clock.days_l = val,
                            0xc => inner_clock.days_h = val,
                            _ => unreachable!(),
                        },
                        Err(_msg) => panic!("poison error!"),
                    }
                }
                self.ram[self.ram_bank_num][addr as usize] = val
            }
        }
    }
}
