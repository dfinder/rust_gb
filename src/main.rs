#[macro_use]
pub mod joypad;
pub mod audio;
pub mod cartridge;
pub mod cpu;
pub mod cpu_state;
pub mod function_table;
pub mod interrupt;
pub mod memory_wrapper;
pub mod registers;
pub mod screen;
use std::{cell::RefCell, fs::File, rc::Rc, time::Duration};

use crate::screen::screen::display_screen;
use audio::audio_controller::AudioController;
use joypad::joypad::Joypad;
use sdl2::{self, event::Event, keyboard::Keycode, pixels::Color};
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Rust Demo",160,144).position_centered().build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let audio_controller = AudioController::new();
    let cartridge = File::open("../Mario.gb").expect("msg");
    let mut joypad: Joypad = joypad::joypad::Joypad::new([
        Keycode::M,
        Keycode::N,
        Keycode::Z,
        Keycode::X,
        Keycode::Down,
        Keycode::Up,
        Keycode::Left,
        Keycode::Right,
    ]);

    let my_cpu = &mut cpu::cpu::CpuStruct::new(
        Rc::new(RefCell::new(joypad)),
        Rc::new(RefCell::new(audio_controller)),
        cartridge,
    );
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown {  keycode, repeat,.. }=>
                {
                    joypad.process_keystrokes(my_cpu, keycode, repeat,true);
                }
                Event::KeyUp{  keycode,  repeat,.. }=>
                {
                    joypad.process_keystrokes(my_cpu, keycode, repeat,false);
                }
                _ => {}
            }
        }
        my_cpu.interpret_command();
        let graphics_state = my_cpu.fetch_graphics();
        //let key_strokes:

        display_screen( &mut canvas, graphics_state);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 238));
    }
}

// Set up window/connectivity with OS
// Read startup data
// Read Cartridge into memory
// Start program counter
// loop
//Read buttons
//Increment program counter
//Execute command at program counter
//Draw Screen
//Doot
//Timers

//
//
//
//frame.clear_color(0.0, 0.0, 1.0, 1.0);

//display::display::display_screen();
