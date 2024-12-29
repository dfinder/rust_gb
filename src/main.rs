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
use std::{cell::RefCell, fs::File, rc::Rc};

use crate::screen::screen::display_screen;
use audio::audio_controller::AudioController;
use glium::{self};
use joypad::joypad::Joypad;
use sdl2::{self};
use winit::{event_loop::EventLoop, keyboard::KeyCode};
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let event_loop = EventLoop::builder().build().expect("test");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let audio_controller = AudioController::new();
    let cartridge = File::open("~/Mario.gb").expect("msg");
    let mut joypad: Joypad = joypad::joypad::Joypad::new([
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::ArrowDown,
        KeyCode::ArrowUp,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ]);

    let my_cpu = &mut cpu::cpu::CpuStruct::new(
        Rc::new(RefCell::new(joypad)),
        Rc::new(RefCell::new(audio_controller)),
        cartridge,
    );
    let _ = event_loop.run(move |event, window_target| {
        let frame = display.draw();
        let my_cpu = my_cpu.interpret_command();
        let graphics_state = my_cpu.fetch_graphics();
        //let key_strokes:
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => window_target.exit(),
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    event,
                    is_synthetic,
                } => {
                    joypad.process_keystrokes(my_cpu, device_id, event, is_synthetic);
                }
                _ => (),
            },

            _ => (),
        };

        display_screen(&display, &frame, graphics_state);
        frame.finish().unwrap();
    });
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
