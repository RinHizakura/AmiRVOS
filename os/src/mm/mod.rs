mod kheap;
mod linked_list_allocator;
mod mapping;
mod page;

pub fn init() {
    kheap::init();
    page::init();
    mapping::init();
}

pub fn test() {
    kheap::test();
    page::test();
}
