use core::mem::size_of;

extern "C" {
   static KERNEL_SIZE: usize;
   static HEAP_START: usize;
}

static mut ALLOC_START: usize = 0;
static mut PAGE_ENTRY: usize = 0;

// assuming that we have at least 128M RAM to be used
const DRAM_SIZE: usize = 0x8000000;
// 4KB page
const PAGE_SIZE: usize = 1 << 12;


macro_rules! align_ceil {
   ($val:expr, $align:expr) => {
         ( ($val + (($align) -1)) & !(($align) - 1))
   };
}

pub struct Page {
    flags: u8,
}

pub fn init() {
    unsafe {
        // total numbers of page entry
        PAGE_ENTRY = (DRAM_SIZE - KERNEL_SIZE) / PAGE_SIZE;
        let tmp = (HEAP_START + PAGE_ENTRY * size_of::<Page,>() );
        ALLOC_START = align_ceil!(tmp, PAGE_SIZE);
        println!("entry num {}", PAGE_ENTRY);
        println!("0x{:X} align to alloc start 0x{:X}", tmp, ALLOC_START);

    }
}
