#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fs::File, rc::Rc};

    use sdl2::{keyboard::Keycode, pixels::Color};

    use crate::{
        audio::audio_controller::AudioController, cpu::cpu::CpuStruct, joypad::joypad::Joypad,
    };

    #[test]
    fn test_1(){
        let mut testing_gb = test_setup("test1.gb");
        testing_gb.test_init();

    }
    fn test_setup(path: &str) -> CpuStruct {
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
        let cartridge = File::open("./tests/test_1.gb").expect("msg");
        let joypad: Joypad = Joypad::new([
            Keycode::M,
            Keycode::N,
            Keycode::Z,
            Keycode::X,
            Keycode::Down,
            Keycode::Up,
            Keycode::Left,
            Keycode::Right,
        ]);
        let wrapped_joypad = Rc::new(RefCell::new(joypad));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.present();
        return CpuStruct::new(wrapped_joypad.clone(), gb_audio, canvas, cartridge);
    
    }
}
