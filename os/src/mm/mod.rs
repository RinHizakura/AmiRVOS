mod kheap;
mod linked_list_allocator;
mod page;

pub fn init() {
    kheap::init();
    page::init();
}

pub fn test() {
    page::test();
}
