#[macro_use]
pub mod joypad;
pub mod audio;
pub mod cartridge;
pub mod cpu;
pub mod memory;
pub mod screen;
use std::{cell::RefCell, cmp::max, fs::File, rc::Rc, thread, time::Duration};

use audio::audio_controller::AudioController;
use cpu::cpu::CpuStruct;
use joypad::joypad::Joypad;
use sdl2::{self, event::Event, keyboard::Keycode, pixels::Color};
use colog;
use sdl2_sys::KeyCode;
fn main() {

    //let mut clog = colog::default_builder();
    colog::init();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let window = video_subsystem.window("Gameboy",160,144).position_centered().build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let gb_audio = AudioController::new(audio_subsystem);
    let cartridge = File::open("../Mario.gb").expect("msg");
    let mut joypad: Joypad = Joypad::new([
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
    let my_cpu = &mut CpuStruct::new(
        wrapped_joypad.clone(),
        gb_audio,
        canvas,
        cartridge,
    );
    let mut clockrate = 16;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown{keycode:Some(Keycode::Comma),..}=>{
                    clockrate = clockrate <<1 
                
                }
                Event::KeyDown{keycode:Some(Keycode::Period),..}=>{
                    clockrate = max(clockrate >>1,1)
                }
                Event::KeyDown {  keycode, repeat:false,.. }=>
                {
                    joypad.process_keystrokes(my_cpu, keycode,true);
                    
                }
                Event::KeyUp{  keycode,  repeat:false,.. }=>
                {
                    joypad.process_keystrokes(my_cpu, keycode, false);
                }
                _ => {}
            }
        }
        my_cpu.interpret_command();
        //thread::sleep(Duration::new(0, 239*clockrate));//239*1024));
        //println!("{:?}", graphics_state);
        //let key_strokes:

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
