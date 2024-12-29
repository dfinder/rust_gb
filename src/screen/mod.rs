//
//use crate::Surface
pub mod oam;
pub mod ppu;
pub mod video_controller;
pub mod vram;
pub mod screen {
    //Have you considered: SDL
    use glium::*;
    //use crate::glium::glutin::surface::ResizeableSurface;
    //use crate::glium::glutin::surface::SurfaceTypeTrait;
    use crate::glium::glutin::surface::*;

    use super::ppu::ppu::PixelColor;
    pub struct VideoController {
        pub lcdc: u8,
        pub stat: u8,
        pub scy: u8,
        pub scx: u8,
        pub ly: u8,
        pub lyc: u8,
        pub dma: u8,
        pub bgp: u8,
        pub obp0: u8,
        pub obp1: u8,
        pub wy: u8,
        pub wx: u8,
    }

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    implement_vertex!(Vertex, position);
    //use crate::glutin::surface::SurfaceTypeTrait;
    pub fn display_screen(
        display: &Display<WindowSurface>,
        frame: &Frame,
        screen: [[PixelColor; 160]; 144],
    ) {
        let vertex1 = Vertex {
            position: [-0.5, -0.5],
        };
        let vertex2 = Vertex {
            position: [0.0, 0.5],
        };
        let vertex3 = Vertex {
            position: [0.5, -0.25],
        };
        let shape = vec![vertex1, vertex2, vertex3];
        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let vertex_shader_src = r#"

        in vec2 position;
    
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

        let fragment_shader_src = r#"

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
        "#;
        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();
    }
}
