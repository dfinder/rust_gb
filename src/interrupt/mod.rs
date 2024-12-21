pub mod interrupt {
    pub enum InterruptType {
        VBlank,
        LCDC,
        Timer,
        Serial, //Unimplemented
        Input,
    }
}
