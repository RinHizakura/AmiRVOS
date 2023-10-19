mod kheap;
mod linked_list_allocator;
pub mod mapping;
pub mod page;

fn test() {
    kheap::test();
    page::test();
    mapping::test();
}

pub fn init() {
    kheap::init();
    page::init();
    mapping::init();

    test();
}
