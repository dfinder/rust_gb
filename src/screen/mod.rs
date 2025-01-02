//
//use crate::Surface
pub mod oam;
pub mod ppu;
pub mod video_controller;
pub mod vram;
pub mod pixelqueue;
pub mod screen {

    use sdl2::{rect::Point, render::Canvas, video::Window,pixels::Color};

    use super::ppu::ppu::GBColor;

    //use crate::glutin::surface::SurfaceTypeTrait;
    pub fn display_screen(
        frame: &mut Canvas<Window>,
        screen: [[GBColor; 160]; 144],
    ) {
        frame.clear();
        for i in 0..144{
            for j in 0..160{
                let screen_color = match &screen[i][j]{
                    GBColor::White => Color::RGB(0xFF, 0xFF, 0xFF),
                    GBColor::LightGrey => Color::RGB(0xb8, 0xb8, 0xb8),
                    GBColor::DarkGrey => Color::RGB(0x68, 0x68, 0x68),
                    GBColor::Black => Color::RGB(0x00, 0x00, 0x00),
                    GBColor::Transparent => Color::RGB(0xff, 0x11, 0x11),
                };
                frame.set_draw_color(screen_color);
                frame.draw_point(Point::new(j as i32, i as i32)).expect("Pixel failed to write");
            }
        }
    }

}
