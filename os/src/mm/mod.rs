mod kheap;
mod page;

pub fn init() {
    kheap::init();
    page::init();
}

pub fn test() {
    page::test();
}
