use crate::lock::Locked;
use crate::config::KERNEL_HEAP_SIZE;
use crate::mm::linked_list_allocator::LinkedListAllocator;

#[global_allocator]
static HEAP_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

pub fn test() {
    use alloc::boxed::Box;

    HEAP_ALLOCATOR.lock().check_order();

    {
        let _a = Box::new("Everyone");
        let _b = Box::new("know");
        let c = Box::new("that");
        let d = Box::new("Amita");
        let e = Box::new("is");
        let _f = Box::new("RinHizakura's");
        let _g = Box::new("wife");

        drop(c);
        drop(e);
        drop(d);
        HEAP_ALLOCATOR.lock().check_order();
    }

    HEAP_ALLOCATOR.lock().check_order();
}
