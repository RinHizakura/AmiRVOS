mod kheap;
mod linked_list_allocator;
pub mod mapping;
pub mod page;

pub fn init() {
    kheap::init();
    page::init();
    mapping::init();
}

pub fn test() {
    kheap::test();
    page::test();
    mapping::test();
}
