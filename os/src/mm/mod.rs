mod kheap;
mod linked_list_allocator;
mod page;

pub fn init() {
    kheap::init();
    page::init();
}

pub fn test() {
    kheap::test();
    page::test();
}
