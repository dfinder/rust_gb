#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::panic::{self, AssertUnwindSafe};
    use std::process::Command;
    use std::{cell::RefCell, fs::{File, OpenOptions}, rc::Rc, thread, time::Duration};

    use log::LevelFilter;
    use sdl2::{EventPump, event::Event, keyboard::{Keycode, Scancode}, pixels::Color};

    use crate::{
        audio::audio_controller::AudioController, cpu::cpu::CpuStruct, joypad::joypad::Joypad,
    };

    #[test]
    fn test() {
        let mut testing_gb = test_generic("./tests/01-special.gb");
      
        let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)      // overwrite each run
        // .append(true)     // use this instead if you want to append
        .open("./logs/01-special.log").expect("Testing");

    // Setup fern logger
         fern::Dispatch::new()
        // Set global log level
        .level(LevelFilter::Trace)
        // Format log messages: just the message, no timestamp, no level
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message))
        })
        // Write to both stdout and file
        .chain(log_file)
        // Apply as global logger
        .apply().unwrap();
        testing_gb.test_init();
        let clockrate = 1;
        
        'running: loop {

            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                testing_gb.on_clock()
            }));

            match result{
                Ok(e)=> (),
                Err(err)=>break 'running

            }

                
            
            thread::sleep(Duration::new(0, 239*clockrate));//239*1024));
            //println!("{:?}", graphics_state);
            //let key_strokes:
        };
        let output = Command::new("../gameboy-doctor/gameboy-doctor")
        .arg("./logs/01-special.gb")
        .arg("cpu_instrs")
        .arg("1")
        .output()
        .expect("Failed to execute command");
        let _ = io::stdout().write_all(&output.stdout);

    }
    fn test_generic(_gb_path: &str) -> CpuStruct {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let window = video_subsystem
            .window("Gameboy", 160, 144)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let gb_audio = AudioController::new(audio_subsystem);
        let event_pump = sdl_context.event_pump().unwrap();
        let wrapped_pump: Rc<RefCell<EventPump>> = Rc::new(RefCell::new(event_pump));

        let cartridge = File::open(_gb_path).expect("msg");
        let joypad: Joypad = Joypad::new(
            [
                Scancode::M,
                Scancode::N,
                Scancode::Z,
                Scancode::X,
                Scancode::Down,
                Scancode::Up,
                Scancode::Left,
                Scancode::Right,
            ],
            wrapped_pump.clone(),
        );
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.present();
        return CpuStruct::new(joypad, gb_audio, canvas, cartridge);
    }
}
