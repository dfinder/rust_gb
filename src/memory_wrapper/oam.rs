pub mod oam{
    use std::ops::Index;

    impl Index<u16> for OamStruct{
        type Output=u8;
        fn index(&self, index: u16) -> &Self::Output {
            todo!()
        }
    }
    pub struct OamStruct{

    }
}