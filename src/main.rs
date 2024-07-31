pub mod screen;
mod cpu_state;
mod memory_wrapper;
use winit::*;
use glium::*;
use crate::screen::screen::display_screen;
#[macro_use]
pub mod cpu;
pub mod registers;
pub mod memory;
extern crate glium;
fn main() {
    let my_cpu = cpu::cpu::CpuStruct::new();

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

    

    //let event_loop = winit::event_loop::EventLoopBuilder::new().build().expect("event loop building");
    //let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);
    //let mut frame = display.draw();
    //frame.clear_color(0.0, 0.0, 1.0, 1.0);
    //display_screen(&display, &frame);
    //frame.finish().unwrap();
    //let _ = event_loop.run(move |event, window_target| {
    //    match event {
    //        winit::event::Event::WindowEvent { event, .. } => match event {
    //            winit::event::WindowEvent::CloseRequested => window_target.exit(),
    //            _ => (),
    //        },
    //        _ => (),
    //    };
    //});

    //display::display::display_screen();
}
