use core::ptr::null_mut;

extern "C" {
    static KERNEL_END: usize;
}

// Page struct flag init with zero to represent a free page
static mut PAGE_STRUCT: [Page; PAGE_ENTRY] = [Page { flags: 0 }; PAGE_ENTRY];

// DRAM start from 0x80000000
const DRAM_BASE: usize = 0x8000_0000;
// Assuming that we have at least 128M RAM to be used
const DRAM_SIZE: usize = 0x800_0000;
// 4KB page
const PAGE_SIZE: usize = 1 << 12;
// The bottom memory address of HEAP to be accessed
// - Warning: we naively assume our kernel size won't exceed 64 KB, this
//   assumption help our code become cleaner
const LOW_MEMORY: usize = 0x8000_0000 + 0x10000;
// The top memory address of HEAP to be accessed
const HIGH_MEMORY: usize = 0x8000_0000 + DRAM_SIZE;
// Number of page entry availibled
const PAGE_ENTRY: usize = (HIGH_MEMORY - LOW_MEMORY) / PAGE_SIZE;

#[derive(Copy, Clone)]
pub struct Page {
    flags: u8,
}

impl Page {
    pub fn is_alloc(&self) -> bool {
        return (self.flags & 1) == 1;
    }

    pub fn is_first(&self) -> bool {
        return (self.flags & !1) != 0;
    }

    pub fn set(&mut self) {
        self.flags = 1;
    }

    pub fn clear(&mut self) {
        self.flags = 0;
    }
}

pub fn init() {
    unsafe {
        assert!(KERNEL_END < LOW_MEMORY);
        println!("{:X} {:X}", KERNEL_END, LOW_MEMORY);
    }
}

/* FIXME: buddy allocator will be needed for better management as my future work
 */
pub fn alloc(order: usize) -> *mut u8 {
    // only 2^n pages allocation is availible
    let pages = 1 << order;

    unsafe {
        for i in 0..PAGE_ENTRY {
            let mut found = false;

            if !PAGE_STRUCT[i].is_alloc() {
                found = true;

                // make sure the following page are also freed for contiguous allocation
                for j in i + 1..i + pages {
                    if PAGE_STRUCT[j].is_alloc() {
                        found = false;
                    }
                }
            }

            if found {
                for j in i..i + pages {
                    PAGE_STRUCT[j].set();
                }
                // trickly record the order+1 of page in flag
                PAGE_STRUCT[i].flags |= ((order + 1) << 1) as u8 & !1;
                return (LOW_MEMORY + i * PAGE_SIZE) as *mut u8;
            }
        }
    }

    return null_mut();
}

pub fn free(ptr: *mut u8) {
    let addr = ptr as usize;

    // refuse the invalid address
    if ptr.is_null() || (addr < LOW_MEMORY) || (addr >= HIGH_MEMORY) {
        return;
    }

    unsafe {
        let idx = (addr - LOW_MEMORY) / PAGE_SIZE;
        let order = (PAGE_STRUCT[idx].flags & !1) >> 1;

        // makr sure the 'ptr' point to the first allocaed block
        if !PAGE_STRUCT[idx].is_alloc() || order == 0 {
            return;
        }
        // note that the record value is 'order + 1'
        let pages = 1 << (order - 1);

        for i in idx..idx + pages {
            assert!(PAGE_STRUCT[i].flags & 1 == 1);
            PAGE_STRUCT[i].clear();
        }
    }
}
