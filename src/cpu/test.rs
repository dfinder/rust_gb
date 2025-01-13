#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fs::File, rc::Rc};

    use sdl2::{
        keyboard::{Keycode, Scancode},
        pixels::Color,
        EventPump,
    };

    use crate::{
        audio::audio_controller::AudioController, cpu::cpu::CpuStruct, joypad::joypad::Joypad,
    };

    #[test]
    fn test_1() {
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
        let event_pump = sdl_context.event_pump().unwrap();
        let wrapped_pump: Rc<RefCell<EventPump>> = Rc::new(RefCell::new(event_pump));

        let cartridge = File::open("./tests/test_1.gb").expect("msg");
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
