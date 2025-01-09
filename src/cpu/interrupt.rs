pub mod interrupt {
    pub enum Interrupt {
        VBlank,
        LCDC,
        Timer,
        Serial, //Unimplemented
        Input,
    }
}
