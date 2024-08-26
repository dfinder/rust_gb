pub mod exram{
    use std::ops::Index;


    pub struct ExternalRam{
        
    }
    impl Index<u16> for ExternalRam{
        type Output=u8;
        fn index(&self, index: u16) -> &Self::Output {
            todo!()
        }
    }
}