#[macro_use]
pub mod joypad;
pub mod cpu;
pub mod registers;
pub mod memory;
pub mod function_table;
pub mod interrupt;
pub mod screen;
pub mod vscreen;
pub mod cpu_state;
pub mod memory_wrapper;
use joypad::joypad::Joypad;
use winit::{event_loop::EventLoopBuilder, keyboard::KeyCode};
use crate::screen::screen::display_screen;
use glium::*;
extern crate glium;
fn main() {
    let mut my_cpu = cpu::cpu::CpuStruct::new();
    let mut graphics_state:&[u8;8192]; //This doesn't actually have to be a borrow
    let event_loop = EventLoopBuilder::new().build().expect("event loop building");
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    let mut frame = display.draw();
    let mut joypad:Joypad = joypad::joypad::Joypad::new([KeyCode::KeyM, KeyCode::KeyN,KeyCode::KeyZ, KeyCode::KeyX, KeyCode::ArrowDown,KeyCode::ArrowUp,KeyCode::ArrowLeft,KeyCode::ArrowRight]);
    loop{
        my_cpu.interpret_command();
        graphics_state = my_cpu.fetch_graphics();
        
        let _ = event_loop.run(move |event, window_target| {
            match event {
                    winit::event::Event::WindowEvent { event, .. } => match event {
                        winit::event::WindowEvent::CloseRequested => window_target.exit(),
                        winit::event::WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                            joypad.process_keystrokes(&mut &my_cpu,device_id,event,is_synthetic);
                        }
                    
                        _ => (),
                     },
                     
                _ => (),
            };}
         );
        display_screen(&display, &frame,graphics_state);
        frame.finish().unwrap();
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
}
