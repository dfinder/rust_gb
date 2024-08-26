pub mod video_controller{
    pub struct VideoController{
        pub lcdc: u8,//LCD control
        pub stat: u8, 
        pub scy: u8,
        pub scx: u8,
        pub ly: u8 ,
        pub lyc: u8,
        pub dma: u8,
        pub bgp: u8, 
        pub obp0: u8,
        pub obp1: u8,
        pub wy: u8,
        pub wx: u8,
    }
}