pub mod screen;
mod cpu_state;
mod memory_wrapper;
pub mod function_table;
use winit::*;
use cpu::cpu::CpuStruct;
use glium::*;
use function_table::function_table::FunFind;
use crate::screen::screen::display_screen;
#[macro_use]
pub mod cpu;
pub mod registers;
pub mod memory;
extern crate glium;
fn main() {
    let function_lookup = [
            //Block 1,
            FunFind::fun_find(0xff,0x00,CpuStruct::nop,1), //done
            FunFind::fun_find(0xcf,0x01,CpuStruct::ldi_r16,3),  //done
            FunFind::fun_find(0xcf,0x02,CpuStruct::str_acc_rmem,2),
            FunFind::fun_find(0xcf,0x03,CpuStruct::inc_r16,2),//done
            FunFind::fun_find(0xc7,0x04,CpuStruct::inc_r8,1),//done
            FunFind::fun_find(0xc7,0x05,CpuStruct::dec_r8,1),//done
            FunFind::fun_find(0xc7,0x06,CpuStruct::ldi_r8,2), //done
            FunFind::fun_find(0xff,0x07,CpuStruct::rlc,1),//done
            FunFind::fun_find(0xff,0x08,CpuStruct::ld_imm_sp,5),//done
            FunFind::fun_find(0xcf,0x09,CpuStruct::add_hl,2),//
            FunFind::fun_find(0xcf,0x0a,CpuStruct::ld_acc_addr,5),//done
            FunFind::fun_find(0xcf,0x0b,CpuStruct::dec_r16,2),//done
            FunFind::fun_find(0xff,0x0f,CpuStruct::rrc,1),//done
            FunFind::fun_find(0xff,0x1f,CpuStruct::rr,1),//done
            FunFind::fun_find(0xff,0x2f,CpuStruct::cpl,1),//done
            FunFind::fun_find(0xff,0x3f,CpuStruct::ccf,1),//done
            FunFind::fun_find(0xff,0x17,CpuStruct::rl,1),//done
            FunFind::fun_find(0xff,0x18,CpuStruct::jr_imm,3),//?
            FunFind::fun_find_w(0xe7,0x20,CpuStruct::jr_cond,3,5),//
            FunFind::fun_find(0xff,0x27,CpuStruct::daa,1),//?
            FunFind::fun_find(0xff,0x37,CpuStruct::scf,1),//done
            FunFind::fun_find(0xff,0x10,CpuStruct::stop,0),//?   
            FunFind::fun_find(0xff,0x76,CpuStruct::halt,0),//?
            FunFind::fun_find_w(0xc0,0x40,CpuStruct::ld_r8_r8,1,2),
            FunFind::fun_find_w(0xf8,0x80,CpuStruct::add,1,2),
            FunFind::fun_find_w(0xf8,0x88,CpuStruct::adc,1,2),
            FunFind::fun_find_w(0xf8,0x90,CpuStruct::sub,1,2),
            FunFind::fun_find_w(0xf8,0x98,CpuStruct::subc,1,2),
            FunFind::fun_find_w(0xf8,0xa0,CpuStruct::and,1,2),
            FunFind::fun_find_w(0xf8,0xa8,CpuStruct::xor,1,2),
            FunFind::fun_find_w(0xf8,0xb0,CpuStruct::or,1,2),
            FunFind::fun_find_w(0xf8,0xb8,CpuStruct::cp,1,2),
            FunFind::fun_find_w(0xff,0xc6,CpuStruct::add,1,2),
            FunFind::fun_find_w(0xff,0xce,CpuStruct::adc,1,2),
            FunFind::fun_find_w(0xff,0xd6,CpuStruct::sub,1,2),
            FunFind::fun_find_w(0xff,0xde,CpuStruct::subc,1,2),
            FunFind::fun_find_w(0xff,0xe6,CpuStruct::and,1,2),
            FunFind::fun_find_w(0xff,0xee,CpuStruct::xor,1,2),
            FunFind::fun_find_w(0xff,0xf6,CpuStruct::or,1,2),
            FunFind::fun_find_w(0xff,0xfe,CpuStruct::cp,1,2),
            FunFind::fun_find_w(0xe7,0xc0,CpuStruct::ret_cond,5,2),
            FunFind::fun_find(0xff,0xc9,CpuStruct::ret,4),
            FunFind::fun_find(0xff,0xd9,CpuStruct::reti,4),
            FunFind::fun_find_w(0xe7,0xc2,CpuStruct::jp_cond_imm,3,2),
            FunFind::fun_find(0xff,0xc3,CpuStruct::jp_imm,4),
            FunFind::fun_find(0xff,0xc9,CpuStruct::jp_hl,1),
            FunFind::fun_find_w(0xe7,0xc4,CpuStruct::call_cond,6,3),
            FunFind::fun_find_w(0xff,0xcd,CpuStruct::call_imm,6,3),
            FunFind::fun_find(0xe7,0xc7,CpuStruct::rst,4),
            FunFind::fun_find(0xcf,0xc1,CpuStruct::pop,3),
            FunFind::fun_find(0xcf,0xc5,CpuStruct::push,4),
            FunFind::fun_find(0xff,0xcb,CpuStruct::nop,1),
            FunFind::fun_find(0xff,0xe2,CpuStruct::str_c,1), //We're storing if we're mapping to memory, we're loadingif we'r 
            FunFind::fun_find(0xff,0xe0,CpuStruct::str_imm8,2),
            FunFind::fun_find(0xff,0xea,CpuStruct::str_imm16,3),

            FunFind::fun_find(0xff,0xf0,CpuStruct::ld_imm8,2),
            FunFind::fun_find(0xff,0xf2,CpuStruct::ld_c,1),
            FunFind::fun_find(0xff,0xfa,CpuStruct::ld_imm16,3),
            FunFind::fun_find(0xff,0xe8,CpuStruct::add_sp_imm8,4), //
            FunFind::fun_find(0xff,0xf8,CpuStruct::ld_hl_imm8,3),
            FunFind::fun_find(0xff,0xf9,CpuStruct::ld_sp_hl,2),
            FunFind::fun_find(0xff,0xf3,CpuStruct::di,1),
            FunFind::fun_find(0xff,0xfb,CpuStruct::ei,1)
        ];

        let cb_block_lookup: [FunFind; 11]=[
            FunFind::fun_find_w(0xf8,0x00,CpuStruct::rlc,2,4),
            FunFind::fun_find_w(0xf8,0x08,CpuStruct::rrc,2,4),
            FunFind::fun_find_w(0xf8,0x10,CpuStruct::rl,2,4),
            FunFind::fun_find_w(0xf8,0x18,CpuStruct::rr,2,4),
            FunFind::fun_find_w(0xf8,0x20,CpuStruct::sla,2,4),
            FunFind::fun_find_w(0xf8,0x28,CpuStruct::sra,2,4),
            FunFind::fun_find_w(0xf8,0x30,CpuStruct::swap,2,4),
            FunFind::fun_find_w(0xf8,0x38,CpuStruct::srl,2,4),
            FunFind::fun_find_w(0xc0,0x40,CpuStruct::bit,2,3),
            FunFind::fun_find_w(0xc0,0x80,CpuStruct::res,2,4),
            FunFind::fun_find_w(0xc0,0x90,CpuStruct::set,2,4)
        ];
    let mut my_cpu = cpu::cpu::CpuStruct::new();
    loop{
        my_cpu.interpret_command();
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
