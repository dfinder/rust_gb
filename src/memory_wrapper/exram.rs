pub mod exram {
    use crate::memory_wrapper::memory_wrapper::AsMemory;

    pub struct ExRam {
        //Determined by cart type!
    }
    impl ExRam {
        pub fn new() -> Self {
            ExRam {}
        }
    }
    impl AsMemory for ExRam {
        fn memory_map(&mut self, addr: u16) -> u8 {
            todo!()
        }

        fn memory_write(&mut self, addr: u16, val: u8) {
            todo!()
        }
    }
}
